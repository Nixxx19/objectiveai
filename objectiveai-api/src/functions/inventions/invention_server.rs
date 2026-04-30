use std::borrow::Cow;
use std::sync::{Arc, Mutex, RwLock};

use dashmap::DashMap;
use futures::{FutureExt, Stream};
use rmcp::{
    Peer, RoleServer, ServerHandler,
    handler::server::router::tool::{ToolRoute, ToolRouter},
    handler::server::tool::ToolCallContext,
    model::{
        CallToolRequestParams, CallToolResult, ClientJsonRpcMessage, Content, Implementation,
        InitializeRequestParams, InitializeResult, ProtocolVersion, ServerCapabilities,
        ServerInfo, ServerJsonRpcMessage, ServerNotification, Tool, ToolListChangedNotification,
    },
    service::RequestContext,
    transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService,
        session::{
            ServerSseMessage, SessionId, SessionManager,
            local::{LocalSessionManager, LocalSessionManagerError},
        },
    },
};
use serde_json::Value;
use tokio::net::TcpListener;
use tokio::sync::OnceCell;
use tokio_util::sync::{CancellationToken, DropGuard};

use objectiveai::functions::inventions::InventionTool;

/// Custom HTTP header carrying the tenant id from the orchestrator
/// through the proxy to this server. The agent client adds it to its
/// `X-MCP-Headers` map; the proxy forwards it on every upstream
/// request; the server uses it to look up the right tenant's tool set
/// for each MCP request.
pub const TENANT_HEADER: &str = "X-Invention-Session-Id";

/// One in-flight invention's slot inside the shared server. Holds its
/// own tool router and its own list of rmcp peers so per-step
/// `tools/list_changed` broadcasts only reach the inventions that
/// actually need to refresh.
#[derive(Clone)]
struct Tenant {
    tool_router: Arc<RwLock<ToolRouter<InventionMcp>>>,
    peers: Arc<Mutex<Vec<Peer<RoleServer>>>>,
}

impl Tenant {
    fn new(tools: Vec<InventionTool>) -> Self {
        Self {
            tool_router: Arc::new(RwLock::new(build_router(tools))),
            peers: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

/// Process-wide lazy spawner for the shared invention MCP server. Mirrors
/// [`crate::agent::completions::ProxySpawner`] — first caller to
/// [`Self::get`] races on the `OnceCell` and binds a single TCP port +
/// spawns one tokio task; everyone else piggybacks on the same handle.
pub struct InventionServerSpawner {
    cell: OnceCell<Arc<InventionServerHandle>>,
    /// Optional runtime handle anchoring the server task. `None` =
    /// `tokio::spawn` against the ambient runtime (production: one
    /// long-lived runtime). `Some` is for tests where the ambient
    /// runtime is per-`#[tokio::test]` and would drop the task.
    handle: Option<tokio::runtime::Handle>,
}

impl Default for InventionServerSpawner {
    fn default() -> Self {
        Self::new()
    }
}

impl InventionServerSpawner {
    pub fn new() -> Self {
        Self {
            cell: OnceCell::new(),
            handle: None,
        }
    }

    /// Same as `new`, but the server's listener task is spawned on the
    /// supplied runtime handle so it survives even after the caller's
    /// runtime drops. Required in `#[tokio::test]` harnesses where each
    /// test owns its own runtime.
    pub fn new_with_handle(handle: tokio::runtime::Handle) -> Self {
        Self {
            cell: OnceCell::new(),
            handle: Some(handle),
        }
    }

    /// Boot the shared server on first call; return the existing handle
    /// on every subsequent call.
    pub async fn get(&self) -> std::io::Result<Arc<InventionServerHandle>> {
        self.cell
            .get_or_try_init(|| async {
                InventionServerHandle::spawn(self.handle.clone()).await
            })
            .await
            .map(Arc::clone)
    }
}

/// The live, single-port-per-process invention MCP server. Each
/// in-flight invention gets a [`Tenant`] entry in `tenants`; per-tenant
/// tool sets are routed by reading the [`TENANT_HEADER`] header off the
/// inbound HTTP request inside the rmcp [`ServerHandler`].
pub struct InventionServerHandle {
    url: String,
    tenants: Arc<DashMap<String, Tenant>>,
    _shutdown: DropGuard,
    _server_handle: tokio::task::AbortHandle,
}

impl InventionServerHandle {
    async fn spawn(handle: Option<tokio::runtime::Handle>) -> std::io::Result<Arc<Self>> {
        let ct = CancellationToken::new();
        let tenants: Arc<DashMap<String, Tenant>> = Arc::new(DashMap::new());

        let mcp = InventionMcp {
            tenants: tenants.clone(),
        };

        let (port_rx, server_handle) = build_and_spawn_server(mcp, ct.clone(), handle);
        let port = port_rx
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        Ok(Arc::new(Self {
            url: format!("http://127.0.0.1:{}/mcp", port),
            tenants,
            _shutdown: ct.drop_guard(),
            _server_handle: server_handle,
        }))
    }

    /// Register a new invention tenant on the shared server. Returns a
    /// session token; drop the token when the invention completes to
    /// free the tenant slot.
    pub fn register(self: &Arc<Self>, initial_tools: Vec<InventionTool>) -> InventionSession {
        let id = format!("inv-{}", uuid::Uuid::new_v4().simple());
        self.tenants.insert(id.clone(), Tenant::new(initial_tools));
        InventionSession {
            id,
            handle: Arc::clone(self),
        }
    }
}

/// Per-invention session token. Owns one [`Tenant`] slot inside the
/// shared [`InventionServerHandle`]; [`Drop`] removes the slot.
pub struct InventionSession {
    id: String,
    handle: Arc<InventionServerHandle>,
}

impl InventionSession {
    /// The shared server's URL — the same string for every tenant.
    /// Tenants are disambiguated server-side via the [`TENANT_HEADER`]
    /// the agent client forwards.
    pub fn url(&self) -> String {
        self.handle.url.clone()
    }

    /// Tenant id; the orchestrator forwards this through the proxy
    /// using `X-MCP-Headers` so the InventionServer can look up the
    /// right tool set on every request.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Replace this tenant's tool set and broadcast
    /// `notifications/tools/list_changed` to its rmcp peers only.
    /// Other tenants are unaffected. Dead peers are pruned lazily.
    pub async fn set_tools(&self, tools: Vec<InventionTool>) {
        // Snapshot the tenant under DashMap's read guard, then drop
        // the guard before any await to avoid holding it across one.
        let tenant = match self.handle.tenants.get(&self.id) {
            Some(e) => e.value().clone(),
            None => return, // tenant removed concurrently
        };

        // Swap the router first so any list_tools racing the broadcast
        // still sees fresh tools.
        *tenant.tool_router.write().unwrap() = build_router(tools);

        let peers: Vec<Peer<RoleServer>> = {
            let g = tenant.peers.lock().unwrap();
            g.clone()
        };
        let mut alive: Vec<Peer<RoleServer>> = Vec::with_capacity(peers.len());
        for peer in peers {
            let result = peer
                .send_notification(ServerNotification::ToolListChangedNotification(
                    ToolListChangedNotification::default(),
                ))
                .await;
            if result.is_ok() {
                alive.push(peer);
            }
        }
        // Prune dead peers.
        *tenant.peers.lock().unwrap() = alive;
    }
}

impl Drop for InventionSession {
    fn drop(&mut self) {
        self.handle.tenants.remove(&self.id);
    }
}

/// rmcp [`ServerHandler`] for the shared invention server. One clone
/// per HTTP session (rmcp's `StreamableHttpService::new` factory pattern);
/// every clone references the same `tenants` map and routes inbound
/// requests by the [`TENANT_HEADER`] HTTP header.
#[derive(Clone)]
struct InventionMcp {
    tenants: Arc<DashMap<String, Tenant>>,
}

impl InventionMcp {
    /// Look up the tenant for this request from the [`TENANT_HEADER`]
    /// header (injected into request extensions by rmcp's
    /// `streamable_http_server::tower::handle_post`). Returns `None`
    /// if the header is missing, malformed, or names an unknown
    /// tenant — the handler will produce an empty / no-op response in
    /// that case rather than 500.
    fn tenant_for(&self, context: &RequestContext<RoleServer>) -> Option<Tenant> {
        let parts = context.extensions.get::<axum::http::request::Parts>()?;
        let id = parts.headers.get(TENANT_HEADER)?.to_str().ok()?;
        self.tenants.get(id).map(|e| e.value().clone())
    }
}

/// `LocalSessionManager` wrapper whose `close_session` is a no-op so rmcp's
/// auto-cleanup (in `streamable_http_server::tower::handle_post`, after
/// `service.waiting().await` returns) doesn't drop the session table
/// entry when the SSE transport briefly idles between agent step calls.
///
/// Sessions need to outlive transient stream gaps that happen between
/// successive `create_streaming` invocations. rmcp's default behavior
/// would tear down the session as soon as the per-request stream task
/// ends, even though the upstream proxy connection is still in use.
/// All other methods delegate. Sessions are freed wholesale when the
/// `InventionServerHandle` task is aborted on `Drop`.
#[derive(Default)]
struct NoCloseSessionManager {
    inner: LocalSessionManager,
}

impl SessionManager for NoCloseSessionManager {
    type Error = LocalSessionManagerError;
    type Transport = <LocalSessionManager as SessionManager>::Transport;

    fn create_session(
        &self,
    ) -> impl std::future::Future<Output = Result<(SessionId, Self::Transport), Self::Error>> + Send
    {
        self.inner.create_session()
    }
    fn initialize_session(
        &self,
        id: &SessionId,
        message: ClientJsonRpcMessage,
    ) -> impl std::future::Future<Output = Result<ServerJsonRpcMessage, Self::Error>> + Send
    {
        self.inner.initialize_session(id, message)
    }
    fn has_session(
        &self,
        id: &SessionId,
    ) -> impl std::future::Future<Output = Result<bool, Self::Error>> + Send {
        self.inner.has_session(id)
    }
    fn close_session(
        &self,
        _id: &SessionId,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send {
        // Intentional no-op — see type-level docs.
        std::future::ready(Ok(()))
    }
    fn create_stream(
        &self,
        id: &SessionId,
        message: ClientJsonRpcMessage,
    ) -> impl std::future::Future<
        Output = Result<impl Stream<Item = ServerSseMessage> + Send + Sync + 'static, Self::Error>,
    > + Send {
        self.inner.create_stream(id, message)
    }
    fn accept_message(
        &self,
        id: &SessionId,
        message: ClientJsonRpcMessage,
    ) -> impl std::future::Future<Output = Result<(), Self::Error>> + Send {
        self.inner.accept_message(id, message)
    }
    fn create_standalone_stream(
        &self,
        id: &SessionId,
    ) -> impl std::future::Future<
        Output = Result<impl Stream<Item = ServerSseMessage> + Send + Sync + 'static, Self::Error>,
    > + Send {
        self.inner.create_standalone_stream(id)
    }
    fn resume(
        &self,
        id: &SessionId,
        last_event_id: String,
    ) -> impl std::future::Future<
        Output = Result<impl Stream<Item = ServerSseMessage> + Send + Sync + 'static, Self::Error>,
    > + Send {
        self.inner.resume(id, last_event_id)
    }
}

/// Build a fresh [`ToolRouter`] from a list of [`InventionTool`]s. Used by
/// both initial tenant construction and per-step tool-set swaps.
#[inline(never)]
fn build_router(tools: Vec<InventionTool>) -> ToolRouter<InventionMcp> {
    let mut tool_router = ToolRouter::<InventionMcp>::new();

    for t in tools {
        let input_schema: serde_json::Map<String, Value> = t.parameters.into_iter().collect();

        let tool_def = Tool {
            name: Cow::Owned(t.name.to_string()),
            title: None,
            description: Some(Cow::Owned(t.description.to_string())),
            input_schema: Arc::new(input_schema),
            output_schema: None,
            annotations: None,
            execution: None,
            icons: None,
            meta: None,
        };

        let call_fn = t.call.clone();
        tool_router.add_route(ToolRoute::new_dyn(
            tool_def,
            move |ctx: ToolCallContext<'_, InventionMcp>| {
                let call_fn = call_fn.clone();
                let arguments = ctx
                    .arguments
                    .clone()
                    .map(Value::Object)
                    .unwrap_or(Value::Object(Default::default()));
                async move {
                    let result = call_fn(arguments).await;
                    match result {
                        Ok(text) => Ok(CallToolResult::success(vec![Content::text(text)])),
                        Err(text) => Ok(CallToolResult::error(vec![Content::text(text)])),
                    }
                }
                .boxed()
            },
        ));
    }

    tool_router
}

impl ServerHandler for InventionMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2025_06_18,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_tool_list_changed()
                .build(),
            server_info: Implementation {
                name: "objectiveai-function-invention".into(),
                title: None,
                version: env!("CARGO_PKG_VERSION").into(),
                description: None,
                icons: None,
                website_url: None,
            },
            instructions: None,
        }
    }

    fn initialize(
        &self,
        request: InitializeRequestParams,
        context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<InitializeResult, rmcp::ErrorData>> + Send + '_
    {
        // Mirror the default impl's peer_info handling.
        if context.peer.peer_info().is_none() {
            context.peer.set_peer_info(request);
        }
        // Capture the peer for this tenant's later `tools/list_changed`
        // notifications. If no tenant matches the header, the peer is
        // dropped — we still return a healthy InitializeResult so rmcp's
        // session lifecycle isn't disrupted, but tool routing for this
        // session will produce empty / not-found responses.
        if let Some(tenant) = self.tenant_for(&context) {
            tenant.peers.lock().unwrap().push(context.peer.clone());
        }
        std::future::ready(Ok(self.get_info()))
    }

    async fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        context: RequestContext<RoleServer>,
    ) -> Result<rmcp::model::ListToolsResult, rmcp::ErrorData> {
        let tools = match self.tenant_for(&context) {
            Some(tenant) => tenant.tool_router.read().unwrap().list_all(),
            None => Vec::new(),
        };
        Ok(rmcp::model::ListToolsResult {
            tools,
            meta: None,
            next_cursor: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let tenant = match self.tenant_for(&context) {
            Some(t) => t,
            None => {
                return Err(rmcp::ErrorData::invalid_params(
                    "no invention tenant header on request",
                    None,
                ));
            }
        };
        // Clone the router under the read lock so the lock guard never
        // spans the `.call(...).await` below — guards are not Send.
        let router = tenant.tool_router.read().unwrap().clone();
        let tcc = ToolCallContext::new(self, request, context);
        router.call(tcc).await
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        // get_tool is sync — we can only consult tenant data we can
        // reach without &context. None is a safe answer (rmcp's default
        // also returns None); the tool_handler validation that calls
        // this is best-effort, and `call_tool` re-routes through
        // tenant_for anyway.
        let _ = name;
        None
    }
}

/// Separate function to prevent rmcp generics from inflating the caller.
#[inline(never)]
fn build_and_spawn_server(
    mcp: InventionMcp,
    ct: CancellationToken,
    runtime_handle: Option<tokio::runtime::Handle>,
) -> (tokio::sync::oneshot::Receiver<u16>, tokio::task::AbortHandle) {
    let (port_tx, port_rx) = tokio::sync::oneshot::channel();
    let ct_child = ct.child_token();

    let task = async move {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let _ = port_tx.send(port);

        let service: StreamableHttpService<InventionMcp, NoCloseSessionManager> =
            StreamableHttpService::new(
                move || Ok(mcp.clone()),
                Default::default(),
                StreamableHttpServerConfig {
                    stateful_mode: true,
                    cancellation_token: ct_child,
                    ..Default::default()
                },
            );

        let router = axum::Router::new().fallback_service(service);
        axum::serve(listener, router).await.ok();
    };

    let handle = match runtime_handle {
        Some(h) => h.spawn(task).abort_handle(),
        None => tokio::spawn(task).abort_handle(),
    };

    (port_rx, handle)
}

#[cfg(test)]
#[path = "invention_server_tests.rs"]
mod tests;
