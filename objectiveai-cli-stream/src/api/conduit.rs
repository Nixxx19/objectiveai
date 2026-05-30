//! `ConduitMcpHandler` — reverse-attach MCP forwarder for the
//! client-app side of the conduit. Hosted by cli-stream; dispatches
//! every `server_request` frame the API pushes down to a real
//! upstream MCP server, caches one `mcp::Connection` per
//! remote-minted `Mcp-Session-Id`, and forwards each upstream
//! `notifications/{tools,resources}/list_changed` back up the WS as
//! a `client_request::Payload::McpListChanged` so the API's
//! `/objectiveai-mcp` GET-SSE stream can re-emit it standard-MCP-shaped.
//!
//! Dispatch on an inbound `server_request`:
//! - **No `Mcp-Session-Id` header (fresh `initialize`).** Dial the
//!   remote with `session_id = None`; the remote mints one and we
//!   key the new `Connection` under it. The synthesized `initialize`
//!   response stamps that id back in the response `Mcp-Session-Id`
//!   header so the proxy adopts it.
//! - **Header present + already in the map.** Reuse the cached
//!   `Connection`.
//! - **Header present + not in the map (continuation resume).** Dial
//!   the remote with `session_id = Some(incoming)`. The SDK handles
//!   the resume branch — many servers don't echo the header back on
//!   resume, so the SDK falls back to the caller's provided id.
//!
//! Then:
//! - `initialize` → synthesize from `connection.initialize_result`
//!   (the SDK already handshook on `connect`). `tools.listChanged` /
//!   `resources.listChanged` are advertised verbatim — the dial-time
//!   `install_list_changed_pump` makes that honest.
//! - notifications (no `id`) → 202 Accepted, no body, never round-trip.
//! - everything else → raw POST through `connection.http_client` +
//!   `connection.url` + `connection.session_id`. Response parsed by
//!   `parse_json_or_sse` (rmcp's `StreamableHttpService` may pick
//!   either shape).
//!
//! `Notifier` is late-bound: the pump needs one, but the `Notifier`
//! is output of `send_streaming_ws(handler, ...)` and the handler is
//! input. The caller constructs the conduit, threads its clone into
//! `send_streaming_ws`, then calls [`ConduitMcpHandler::install_notifier`]
//! on the original handle once the notifier is in hand. Pump closures
//! read the slot at fire time; events that fire before install are
//! dropped (the window is bounded by a few statements at stream
//! startup — see the plan doc).

use dashmap::DashMap;
use indexmap::IndexMap;
use objectiveai_sdk::Notifier;
use objectiveai_sdk::client_objectiveai_mcp::client_request::{
    McpListChanged, McpListChangedKind,
};
use objectiveai_sdk::client_objectiveai_mcp::{server_request, server_response};
use objectiveai_sdk::http::McpHandler;
use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::sync::OnceCell;

/// Header on every API-originated request to the synthetic
/// `/objectiveai-mcp` URL when the agent declared
/// `client_objectiveai_mcp`. Base64url-no-pad JSON `{names, objectiveai_builtins}`.
/// Consumed by [`forward`]'s `tools/list` branch and stripped from
/// the upstream-forwarded headers.
const TOOLS_ALLOWED_HEADER: &str = "X-OBJECTIVEAI-TOOLS-ALLOWED";

struct ConduitState {
    connection: objectiveai_sdk::mcp::Connection,
}

#[derive(Clone)]
pub struct ConduitMcpHandler {
    inner: Arc<Inner>,
}

struct Inner {
    /// Configured remote MCP URL (e.g. `https://mcp.example.com`).
    /// `None` ⇒ MCP isn't configured for this invocation; every
    /// request 501s the same way `objectiveai_sdk::http::RejectHandler`
    /// would.
    mcp_url: Option<String>,
    client: objectiveai_sdk::mcp::Client,
    connections: DashMap<String, Arc<ConduitState>>,
    /// Late-bound: filled by [`ConduitMcpHandler::install_notifier`]
    /// after the WS-creating call returns the notifier. Pump
    /// closures read it at fire time.
    notifier: OnceLock<Notifier>,
    /// Filesystem root for resolving installed plugin/tool manifest
    /// names — used by the `tools/list` filter to recognize
    /// `objectiveai-mcp` built-ins (any returned tool not in this set
    /// is presumed a built-in when the allow-list's
    /// `objectiveai_builtins` flag is set). `None` means filesystem
    /// is unavailable; the `objectiveai_builtins` flag effectively
    /// becomes a no-op (only explicit names match).
    config_base_dir: Option<PathBuf>,
    /// Lazy cache of installed plugin + tool manifest names. Populated
    /// on first `tools/list` arrival with the `objectiveai_builtins`
    /// flag set. Empty `HashSet` when filesystem is unavailable or
    /// nothing is installed.
    installed_names: OnceCell<HashSet<String>>,
}

impl ConduitMcpHandler {
    /// Construct a handler that dials the given URL on first use.
    /// `mcp_url = None` makes every `handle()` call reject with 501.
    /// `config_base_dir` is the filesystem root the CLI consults to
    /// recognize objectiveai-mcp built-ins for the `tools/list`
    /// filter — `None` keeps the filter pure-explicit-names.
    pub fn new(mcp_url: Option<String>, config_base_dir: Option<PathBuf>) -> Self {
        let http = reqwest::Client::builder()
            .build()
            .expect("reqwest::Client::build is infallible without rustls toggles");
        let client = objectiveai_sdk::mcp::Client::new(
            http,
            "objectiveai-cli-stream-conduit".to_string(),
            String::new(),
            String::new(),
            Duration::from_secs(30),
            Duration::from_secs(1),
            Duration::from_secs(1),
            0.5,
            2.0,
            Duration::from_secs(30),
            Duration::from_secs(30),
            Duration::from_secs(60),
        );
        Self {
            inner: Arc::new(Inner {
                mcp_url,
                client,
                connections: DashMap::new(),
                notifier: OnceLock::new(),
                config_base_dir,
                installed_names: OnceCell::new(),
            }),
        }
    }

    /// Install the `Notifier` the list-changed pump uses to push
    /// `McpListChanged` frames up the WS. Idempotent — first set
    /// wins; later calls are no-ops. Call once, after
    /// `send_streaming_ws` returns the notifier and before the proxy
    /// could plausibly have triggered upstream `list_changed` fires.
    pub fn install_notifier(&self, notifier: Notifier) {
        let _ = self.inner.notifier.set(notifier);
    }

    async fn dial(
        &self,
        url: String,
        session_id: Option<String>,
        request_headers: &IndexMap<String, String>,
    ) -> Result<Arc<ConduitState>, objectiveai_sdk::mcp::Error> {
        let connect_headers = sanitize_connect_headers(request_headers);
        let connection = self
            .inner
            .client
            .connect(url, session_id, Some(connect_headers))
            .await?;
        install_list_changed_pump(
            &connection,
            self.inner.clone(),
            connection.session_id.clone(),
        );
        Ok(Arc::new(ConduitState { connection }))
    }
}

impl McpHandler for ConduitMcpHandler {
    async fn handle(&self, request: server_request::Request) -> server_response::Response {
        let id_for_err = request.id.clone();

        let Some(mcp_url) = self.inner.mcp_url.as_ref() else {
            return reject_no_mcp(id_for_err);
        };

        let incoming_session_id: Option<String> = request
            .headers
            .iter()
            .find_map(|(k, v)| {
                k.eq_ignore_ascii_case("mcp-session-id").then(|| v.clone())
            });

        let state = match &incoming_session_id {
            Some(sid) => {
                if let Some(existing) = self.inner.connections.get(sid) {
                    existing.clone()
                } else {
                    let dial_result = self
                        .dial(mcp_url.clone(), Some(sid.clone()), &request.headers)
                        .await;
                    match dial_result {
                        Ok(st) => {
                            self.inner.connections.insert(sid.clone(), st.clone());
                            st
                        }
                        Err(e) => {
                            return conduit_error(id_for_err, format!("connect (resume): {e}"));
                        }
                    }
                }
            }
            None => {
                let dial_result = self.dial(mcp_url.clone(), None, &request.headers).await;
                match dial_result {
                    Ok(st) => {
                        self.inner
                            .connections
                            .insert(st.connection.session_id.clone(), st.clone());
                        st
                    }
                    Err(e) => {
                        return conduit_error(id_for_err, format!("connect: {e}"));
                    }
                }
            }
        };

        match forward(&self.inner, &state, request).await {
            Ok(resp) => resp,
            Err(e) => conduit_error(id_for_err, e.to_string()),
        }
    }
}

/// Wire `set_on_{tools,resources}_list_changed` to fire-and-forget
/// notifier sends. Closures read the late-bound `Notifier` from the
/// `Inner`'s `OnceLock` at fire time — events that fire before
/// `install_notifier` is called are dropped silently.
fn install_list_changed_pump(
    connection: &objectiveai_sdk::mcp::Connection,
    inner: Arc<Inner>,
    mcp_session_id: String,
) {
    let inner_tools = inner.clone();
    let session_tools = mcp_session_id.clone();
    connection.set_on_tools_list_changed(move || {
        let Some(notifier) = inner_tools.notifier.get().cloned() else {
            return;
        };
        let mcp_session_id = session_tools.clone();
        tokio::spawn(async move {
            let _ = notifier
                .notify_list_changed(McpListChanged {
                    mcp_session_id,
                    kind: McpListChangedKind::Tools,
                })
                .await;
        });
    });

    let inner_resources = inner;
    let session_resources = mcp_session_id;
    connection.set_on_resources_list_changed(move || {
        let Some(notifier) = inner_resources.notifier.get().cloned() else {
            return;
        };
        let mcp_session_id = session_resources.clone();
        tokio::spawn(async move {
            let _ = notifier
                .notify_list_changed(McpListChanged {
                    mcp_session_id,
                    kind: McpListChangedKind::Resources,
                })
                .await;
        });
    });
}

/// Hop-by-hop and layer-internal headers don't propagate to MCP.
fn sanitize_connect_headers(
    headers: &IndexMap<String, String>,
) -> IndexMap<String, String> {
    let mut out = headers.clone();
    for k in [
        "Host",
        "host",
        "Content-Length",
        "content-length",
        "Mcp-Session-Id",
        "mcp-session-id",
    ] {
        out.shift_remove(k);
    }
    out
}

async fn forward(
    inner: &Arc<Inner>,
    state: &ConduitState,
    request: server_request::Request,
) -> Result<server_response::Response, ConduitError> {
    let envelope = request.body.clone();

    let rpc_id = envelope.as_ref().and_then(|v| v.get("id")).cloned();
    let rpc_method = envelope
        .as_ref()
        .and_then(|v| v.get("method"))
        .and_then(|m| m.as_str())
        .map(|s| s.to_string());

    // Notifications (no `id`) → 202 with no body; don't round-trip.
    if rpc_id.is_none() {
        return Ok(server_response::Response {
            id: request.id,
            status: 202,
            headers: IndexMap::new(),
            body: None,
        });
    }

    // `initialize`: synthesize from the SDK Connection's cached
    // InitializeResult; don't re-handshake. Stamp the remote-minted
    // session id on the response so the proxy adopts it.
    //
    // `tools.listChanged` / `resources.listChanged` are advertised
    // verbatim — `install_list_changed_pump` forwards each fire up
    // the WS as `client_request::Payload::McpListChanged`, which the
    // API surfaces on its GET-SSE notifications stream so the proxy's
    // `mcp::Connection` to this endpoint sees real MCP list_changed
    // events.
    if rpc_method.as_deref() == Some("initialize") {
        let init_value = serde_json::to_value(&state.connection.initialize_result)
            .map_err(ConduitError::Serialize)?;
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": rpc_id.unwrap(),
            "result": init_value,
        });
        let mut headers = IndexMap::new();
        headers.insert(
            "Mcp-Session-Id".to_string(),
            state.connection.session_id.clone(),
        );
        return Ok(server_response::Response {
            id: request.id,
            status: 200,
            headers,
            body: Some(body),
        });
    }

    // Everything else: raw POST through the Connection.
    let conn = &state.connection;
    let mut req = conn.http_client.post(&conn.url);
    for (k, v) in &request.headers {
        if k.eq_ignore_ascii_case("host")
            || k.eq_ignore_ascii_case("content-length")
            || k.eq_ignore_ascii_case("connection")
            || k.eq_ignore_ascii_case("accept")
            || k.eq_ignore_ascii_case("content-type")
            || k.eq_ignore_ascii_case("mcp-session-id")
            || k.eq_ignore_ascii_case(TOOLS_ALLOWED_HEADER)
        {
            // X-OBJECTIVEAI-TOOLS-ALLOWED is an API↔CLI signal; the
            // upstream MCP server doesn't need to see it.
            continue;
        }
        req = req.header(k, v);
    }
    req = req
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .header("Mcp-Session-Id", &conn.session_id);
    if let Some(body) = envelope.as_ref() {
        req = req.json(body);
    }

    let resp = req.send().await.map_err(ConduitError::Request)?;
    let status = resp.status().as_u16();
    let mut resp_headers = IndexMap::new();
    for (k, v) in resp.headers().iter() {
        if k.as_str().eq_ignore_ascii_case("mcp-session-id")
            || k.as_str().eq_ignore_ascii_case("content-type")
            || k.as_str().eq_ignore_ascii_case("transfer-encoding")
            || k.as_str().eq_ignore_ascii_case("content-length")
        {
            // mcp-session-id: local-layer; the API uses the conduit's
            // real session id stamped elsewhere.
            // content-type: the API re-sets it on body presence.
            // transfer-encoding / content-length: framing headers
            // scoped to the conduit↔objectiveai-mcp TCP connection.
            continue;
        }
        if let Ok(value) = v.to_str() {
            resp_headers.insert(k.as_str().to_string(), value.to_string());
        }
    }
    let resp_text = resp.text().await.map_err(ConduitError::Body)?;
    let mut resp_body = parse_json_or_sse(&resp_text);

    // `tools/list`: if the API stamped `X-OBJECTIVEAI-TOOLS-ALLOWED`,
    // narrow `result.tools[]` to only the entries the agent's
    // `client_objectiveai_mcp` declaration referenced. The API does
    // the validation pass (assert each required tool present);
    // filtering is the CLI's job now.
    if rpc_method.as_deref() == Some("tools/list") {
        if let Some(allowed) = read_tools_allowed_header(&request.headers) {
            if let Some(body) = resp_body.as_mut() {
                apply_tools_filter(inner, body, &allowed).await;
            }
        }
    }

    Ok(server_response::Response {
        id: request.id,
        status,
        headers: resp_headers,
        body: resp_body,
    })
}

/// Decoded `X-OBJECTIVEAI-TOOLS-ALLOWED` payload.
#[derive(Debug, serde::Deserialize)]
struct ToolsAllowed {
    #[serde(default)]
    names: Vec<String>,
    #[serde(default)]
    objectiveai_builtins: bool,
}

fn read_tools_allowed_header(
    headers: &IndexMap<String, String>,
) -> Option<ToolsAllowed> {
    use base64::Engine;
    let raw = headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(TOOLS_ALLOWED_HEADER))?
        .1;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(raw.as_bytes())
        .ok()?;
    serde_json::from_slice::<ToolsAllowed>(&bytes).ok()
}

/// In-place filter on a JSON-RPC `tools/list` response body. Keeps a
/// returned tool iff either:
///
/// - its name matches an explicit `allowed.names` entry exactly OR
///   with a `_<name>` suffix (mirrors the API's existing match
///   tolerance for upstream-namespaced tool names), OR
/// - `allowed.objectiveai_builtins` is set AND the tool's name isn't
///   among the CLI's locally-installed plugin/tool manifests (so it
///   must be an `objectiveai-mcp` built-in).
///
/// Drops everything else. No-op on bodies that don't carry a
/// `result.tools` array.
async fn apply_tools_filter(
    inner: &Arc<Inner>,
    body: &mut serde_json::Value,
    allowed: &ToolsAllowed,
) {
    let Some(tools) = body
        .get_mut("result")
        .and_then(|r| r.get_mut("tools"))
        .and_then(|t| t.as_array_mut())
    else {
        return;
    };

    let installed: Option<&HashSet<String>> = if allowed.objectiveai_builtins {
        Some(inner.installed_names.get_or_init(|| load_installed_names(inner)).await)
    } else {
        None
    };

    tools.retain(|tool| {
        let Some(name) = tool.get("name").and_then(|n| n.as_str()) else {
            return false;
        };
        if allowed.names.iter().any(|declared| {
            name == declared || name.ends_with(&format!("_{declared}"))
        }) {
            return true;
        }
        if let Some(installed) = installed {
            return !installed.contains(name);
        }
        false
    });
}

/// Enumerate installed plugin + tool manifest names under
/// `config_base_dir`. Returns an empty set if the dir is unset or
/// neither directory exists. Used by the `objectiveai_builtins`
/// branch of [`apply_tools_filter`] to recognize built-ins by
/// elimination.
async fn load_installed_names(inner: &Arc<Inner>) -> HashSet<String> {
    let mut names: HashSet<String> = HashSet::new();
    let Some(base_dir) = inner.config_base_dir.clone() else {
        return names;
    };
    let fs = objectiveai_sdk::filesystem::Client::new(
        Some(base_dir),
        None::<String>,
        None::<String>,
    );
    for entry in fs.list_plugins(0, usize::MAX).await {
        names.insert(entry.name);
    }
    for entry in fs.list_tools(0, usize::MAX).await {
        names.insert(entry.name);
    }
    names
}

fn reject_no_mcp(id: String) -> server_response::Response {
    server_response::Response {
        id,
        status: 501,
        headers: IndexMap::new(),
        body: Some(serde_json::json!({
            "jsonrpc": "2.0",
            "id": serde_json::Value::Null,
            "error": {
                "code": -32601,
                "message": "this client has no MCP server configured (pass --mcp-address)",
            },
        })),
    }
}

fn conduit_error(id: String, message: impl Into<String>) -> server_response::Response {
    let message = message.into();
    server_response::Response {
        id,
        status: 502,
        headers: IndexMap::new(),
        body: Some(serde_json::json!({
            "jsonrpc": "2.0",
            "id": serde_json::Value::Null,
            "error": {
                "code": -32603,
                "message": format!("conduit: {message}"),
            },
        })),
    }
}

/// Parses bare JSON; falls back to stripping `data:` prefixes and
/// reparsing for SSE-wrapped responses.
fn parse_json_or_sse(text: &str) -> Option<serde_json::Value> {
    if text.is_empty() {
        return None;
    }
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(text) {
        return Some(v);
    }
    let collected: String = text
        .lines()
        .filter_map(|l| l.strip_prefix("data: ").or_else(|| l.strip_prefix("data:")))
        .collect();
    if collected.is_empty() {
        return None;
    }
    serde_json::from_str::<serde_json::Value>(&collected).ok()
}

#[derive(Debug, thiserror::Error)]
enum ConduitError {
    #[error("forwarding HTTP request failed: {0}")]
    Request(reqwest::Error),
    #[error("reading response body failed: {0}")]
    Body(reqwest::Error),
    #[error("serializing InitializeResult failed: {0}")]
    Serialize(serde_json::Error),
}
