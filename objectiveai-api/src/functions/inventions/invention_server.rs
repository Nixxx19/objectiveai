use std::borrow::Cow;
use std::sync::{Arc, Mutex, RwLock};

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
use tokio_util::sync::CancellationToken;

use objectiveai::functions::inventions::InventionTool;

/// In-process MCP HTTP server that wraps a set of `InventionTool` callables.
///
/// The server lives for the duration of an entire invention. The orchestrator
/// calls [`InventionServer::set_tools`] between invention steps to swap in the
/// tool set for the next step; each swap broadcasts a
/// `notifications/tools/list_changed` to every live MCP session so the proxy
/// invalidates its cache and the next `tools/list` from the agent reflects the
/// new tools.
pub struct InventionServer {
    port: u16,
    _cancel: CancellationToken,
    server_handle: tokio::task::AbortHandle,
    /// Shared with every per-session [`InventionMcp`] handler clone — see
    /// `StreamableHttpService::new`'s factory closure. Holding it here lets
    /// [`InventionServer::set_tools`] swap the tool set without rebuilding the
    /// HTTP server.
    tool_router: Arc<RwLock<ToolRouter<InventionMcp>>>,
    /// Per-session [`Peer`] handles registered by `InventionMcp::initialize`.
    /// `set_tools` walks these to push `tools/list_changed` notifications.
    /// Sessions that have closed return `TransportClosed` from `send_notification`;
    /// we prune those entries lazily on each broadcast.
    peers: Arc<Mutex<Vec<Peer<RoleServer>>>>,
}

#[derive(Clone)]
struct InventionMcp {
    /// Routes get swapped under this lock by [`InventionServer::set_tools`].
    /// Critical sections are short — clone the router under the lock and run
    /// the call after release so the read guard never spans an `.await`.
    tool_router: Arc<RwLock<ToolRouter<Self>>>,
    /// Shared peer registry; appended to in `initialize`.
    peers: Arc<Mutex<Vec<Peer<RoleServer>>>>,
}

/// `LocalSessionManager` wrapper whose `close_session` is a no-op so rmcp's
/// auto-cleanup (in `streamable_http_server::tower::handle_post`, after
/// `service.waiting().await` returns) doesn't drop the session table
/// entry when the SSE transport briefly idles between agent step calls.
///
/// The InventionServer needs sessions to outlive transient stream gaps
/// that can happen between successive `create_streaming` invocations.
/// rmcp's default behavior would tear down the session as soon as the
/// per-request stream task ends, even though the upstream proxy
/// connection (and therefore the session id) is still in use. All other
/// methods delegate. The session table is freed wholesale when the
/// `InventionServer` task is aborted on `Drop`.
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
/// both initial construction and tool-set swap.
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
        // Mirror the default impl's peer_info handling, then capture the
        // peer for later `tools/list_changed` notifications.
        if context.peer.peer_info().is_none() {
            context.peer.set_peer_info(request);
        }
        self.peers.lock().unwrap().push(context.peer.clone());
        std::future::ready(Ok(self.get_info()))
    }

    async fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<rmcp::model::ListToolsResult, rmcp::ErrorData> {
        let tools = self.tool_router.read().unwrap().list_all();
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
        // Clone the router under the read lock so the lock guard never spans
        // the `.call(...).await` below — guards are not Send.
        let router = self.tool_router.read().unwrap().clone();
        let tcc = ToolCallContext::new(self, request, context);
        router.call(tcc).await
    }

    fn get_tool(&self, name: &str) -> Option<Tool> {
        self.tool_router.read().unwrap().get(name).cloned()
    }
}

/// Separate function to prevent rmcp generics from inflating the caller.
#[inline(never)]
fn build_and_spawn_server(
    mcp: InventionMcp,
    ct: CancellationToken,
) -> (tokio::sync::oneshot::Receiver<u16>, tokio::task::AbortHandle) {
    let (port_tx, port_rx) = tokio::sync::oneshot::channel();
    let ct_child = ct.child_token();

    let handle = tokio::spawn(async move {
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
    })
    .abort_handle();

    (port_rx, handle)
}

impl InventionServer {
    pub async fn new(tools: Vec<InventionTool>) -> Self {
        let ct = CancellationToken::new();

        let tool_router = Arc::new(RwLock::new(build_router(tools)));
        let peers: Arc<Mutex<Vec<Peer<RoleServer>>>> = Arc::new(Mutex::new(Vec::new()));
        let mcp = InventionMcp {
            tool_router: tool_router.clone(),
            peers: peers.clone(),
        };

        let (port_rx, server_handle) = build_and_spawn_server(mcp, ct.clone());
        let port = port_rx.await.unwrap();

        Self {
            port,
            _cancel: ct,
            server_handle,
            tool_router,
            peers,
        }
    }

    /// Streamable-HTTP MCP endpoint URL (one entry to add to the proxy's
    /// `X-MCP-Servers` array).
    pub fn url(&self) -> String {
        format!("http://127.0.0.1:{}/mcp", self.port)
    }

    /// Replace the live tool set with `tools` and broadcast
    /// `notifications/tools/list_changed` to every connected session.
    ///
    /// The router is swapped atomically before notifications fire, so the
    /// next `tools/list` the proxy issues against this server is guaranteed
    /// to return the new set. Dead peers (sessions that have closed) are
    /// pruned from the registry as a side effect.
    pub async fn set_tools(&self, tools: Vec<InventionTool>) {
        // Swap the router first so any list_tools racing the broadcast still
        // sees fresh tools.
        *self.tool_router.write().unwrap() = build_router(tools);

        let peers: Vec<Peer<RoleServer>> = {
            let g = self.peers.lock().unwrap();
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
        *self.peers.lock().unwrap() = alive;
    }
}

impl Drop for InventionServer {
    fn drop(&mut self) {
        self.server_handle.abort();
    }
}

#[cfg(test)]
#[path = "invention_server_tests.rs"]
mod tests;
