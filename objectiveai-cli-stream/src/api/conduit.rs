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
use objectiveai_sdk::cli::plugins::PluginOutput;
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
const MCP_CONFIG_HEADER: &str = "X-OBJECTIVEAI-MCP-CONFIG";

struct ConduitState {
    connection: objectiveai_sdk::mcp::Connection,
}

/// One MCP connection the CLI has dialed via a `PLUGIN_MCP_CONNECT`
/// request. Wraps the bare `mcp::Connection` so future commits can
/// grow the per-connection state (proxy routes, request handlers,
/// listener tasks, …) without churning the storage type.
struct PluginMcpState {
    connection: objectiveai_sdk::mcp::Connection,
    /// ws_session_ids that have selected this `(plugin, mcp_name)`
    /// in their most recent `tools/list` (via the `mcp_servers`
    /// field on the `X-OBJECTIVEAI-MCP-CONFIG` header). Read by the
    /// `set_on_{tools,resources}_list_changed` callbacks installed
    /// on `connection` to fan out `McpListChanged` frames per
    /// interested session. Mutated by the diff logic in the
    /// `tools/list` arm.
    interested_sessions: dashmap::DashSet<String>,
}

/// Per-`ws_session_id` state derived from inbound requests. Holds
/// the most recent plugin-mcp selection (for diff-based
/// `interested_sessions` maintenance) and the primary upstream's
/// `mcp_session_id` (recorded the first time `initialize` lands for
/// this ws_session_id, used as the `mcp_session_id` on
/// `McpListChanged` frames fanned out from plugin upstreams).
struct SessionState {
    last_selected: std::sync::Mutex<Vec<(String, String)>>,
    primary_mcp_session_id: OnceLock<String>,
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
    /// MCP connections the CLI has dialed via `PLUGIN_MCP_CONNECT`,
    /// keyed by `(plugin_name, mcp_name)` — the same vocabulary the
    /// API uses on the wire. Populated lazily by the background dial
    /// in `handle_plugin_mcp_connect` (on `mcp::Client::connect`
    /// success). Lives for the lifetime of `Inner`; entries drop
    /// with the WS session, which tears down each `Connection`'s
    /// SSE listener and HTTP stream.
    ///
    /// Populated by the background dial in
    /// `handle_plugin_mcp_connect`. Consumed by `tools/list`
    /// aggregation (when the per-session selection lists this
    /// `(plugin, mcp_name)`) and by the per-connection
    /// `set_on_{tools,resources}_list_changed` callbacks installed
    /// at dial time, which fan list_changed events out to every
    /// `ws_session_id` in `interested_sessions`.
    plugin_mcp_connections: DashMap<(String, String), Arc<PluginMcpState>>,
    /// Per-`ws_session_id` `SessionState`, lazily created on first
    /// inbound request that carries an `X-OBJECTIVEAI-RESPONSE-ID`
    /// (or first `initialize` we see). Tracks the most recent
    /// plugin-mcp selection and the primary upstream's
    /// `mcp_session_id` for `list_changed` routing.
    sessions: DashMap<String, Arc<SessionState>>,
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
                plugin_mcp_connections: DashMap::new(),
                sessions: DashMap::new(),
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

    /// Handle a `method = "PLUGIN_MCP_CONNECT"` server_request:
    /// verify the plugin is installed and declares the named MCP
    /// server in its manifest, spawn its binary with `mcp <name>
    /// begin`, capture the first `Mcp { url }` notification it emits
    /// on stdout, then **dial that URL in a background task and
    /// discard the connection** before returning a 200 ack (no body).
    ///
    /// The API never sees the URL. Errors before the URL is captured
    /// (deserialization, missing plugin, missing manifest entry,
    /// spawn failure, plugin error notification, timeout, no-mcp-
    /// emit) map to 4xx/5xx with a `{"error": "..."}` body. The
    /// background dial's outcome is not reported.
    ///
    /// The spawned plugin process keeps running after the URL is
    /// captured — it IS the MCP server. A background task drains
    /// its stdout/stderr so the pipes don't fill up. A second
    /// background task dials the MCP connection and drops it.
    async fn handle_plugin_mcp_connect(
        &self,
        request: server_request::Request,
    ) -> server_response::Response {
        let id = request.id;

        #[derive(serde::Deserialize)]
        struct Params {
            plugin_name: String,
            mcp_name: String,
        }
        let Some(body) = request.body else {
            return error_response(id, 400, "PLUGIN_MCP_BEGIN: missing request body");
        };
        let Params { plugin_name, mcp_name } = match serde_json::from_value(body) {
            Ok(p) => p,
            Err(e) => {
                return error_response(
                    id,
                    400,
                    format!("PLUGIN_MCP_CONNECT: invalid body: {e}"),
                );
            }
        };

        // Idempotency gate: if we already have a live connection for
        // this (plugin, mcp_name), skip the manifest verify, plugin
        // spawn, and dial entirely. Re-dialing would waste a plugin
        // process and replace a working connection for no reason.
        // The lookup uses an owned-key clone because DashMap doesn't
        // borrow tuple keys; cheap (two short Strings).
        if self
            .inner
            .plugin_mcp_connections
            .contains_key(&(plugin_name.clone(), mcp_name.clone()))
        {
            return ok_response(id);
        }

        let Some(base_dir) = self.inner.config_base_dir.clone() else {
            return error_response(
                id,
                500,
                "PLUGIN_MCP_BEGIN: filesystem unavailable (no config_base_dir)",
            );
        };
        let fs = objectiveai_sdk::filesystem::Client::new(
            Some(base_dir),
            None::<String>,
            None::<String>,
        );

        let Some(plugin) = fs.get_plugin(&plugin_name).await else {
            return error_response(
                id,
                404,
                format!("plugin {plugin_name:?} not installed"),
            );
        };

        // Manifest verification gate — run BEFORE any subprocess
        // spawn so a bogus mcp_name fails fast.
        if !plugin
            .manifest
            .mcp_servers
            .iter()
            .any(|s| s.name == mcp_name)
        {
            return error_response(
                id,
                404,
                format!(
                    "plugin {plugin_name:?} manifest does not declare mcp server {mcp_name:?}"
                ),
            );
        }

        let Some(exe) = fs.resolve_plugin(&plugin_name).await else {
            return error_response(
                id,
                404,
                format!("plugin {plugin_name:?} binary not found"),
            );
        };

        let mut child = match tokio::process::Command::new(&exe)
            .arg("mcp")
            .arg(&mcp_name)
            .arg("begin")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(e) => {
                return error_response(
                    id,
                    500,
                    format!("PLUGIN_MCP_BEGIN: spawn failed: {e}"),
                );
            }
        };

        let stdout = child.stdout.take().expect("stdout was piped");
        let stderr = child.stderr.take().expect("stderr was piped");

        use tokio::io::AsyncBufReadExt;
        let reader = tokio::io::BufReader::new(stdout);
        let mut lines = reader.lines();

        let timeout = std::time::Duration::from_secs(30);
        let begin_result = tokio::time::timeout(timeout, async {
            loop {
                let line = match lines.next_line().await {
                    Ok(Some(l)) => l,
                    Ok(None) => {
                        return Err::<objectiveai_sdk::cli::output::Mcp, String>(
                            "plugin exited without emitting mcp{url}".into(),
                        );
                    }
                    Err(e) => return Err(format!("plugin stdout read error: {e}")),
                };
                let out = match serde_json::from_str::<PluginOutput>(&line) {
                    Ok(o) => o,
                    Err(_) => continue,
                };
                match out {
                    PluginOutput::Mcp(mcp) => return Ok(mcp),
                    PluginOutput::Error(err) => {
                        return Err(format!(
                            "plugin emitted error: {}",
                            err.message
                        ));
                    }
                    // Other notifications / commands before the
                    // mcp announcement are tolerated and skipped —
                    // the host's plugin-MCP-begin path is one-shot
                    // on the Mcp variant.
                    PluginOutput::Notification(_)
                    | PluginOutput::Command { .. } => {}
                }
            }
        })
        .await;

        // Hand off child + remaining IO to a detached task so the
        // plugin keeps running after we've captured (or failed to
        // capture) the URL. Drains stdout and forwards stderr so
        // pipe buffers don't fill up.
        tokio::spawn(async move {
            let stderr_task = tokio::spawn(forward_stderr(stderr));
            while let Ok(Some(_)) = lines.next_line().await {
                // discard
            }
            let _ = stderr_task.await;
            let _ = child.wait().await;
        });

        match begin_result {
            Ok(Ok(mcp)) => {
                // Dial the captured URL in the background; on
                // success, store the `Connection` in
                // `inner.plugin_mcp_connections` keyed by
                // `(plugin_name, mcp_name)`. This is the
                // "non-blocking" leg: the API gets its ack
                // immediately after URL capture and isn't gated on
                // the upstream MCP initialize round-trip. Dial
                // failures are silent at this layer — the API was
                // already acked; future commits will surface them
                // through a separate channel if needed.
                //
                // Race: two simultaneous connects for the same key
                // both pass the idempotency gate above, both dial,
                // and the later insert overwrites the earlier (the
                // displaced `Arc` drops, killing its `Connection`
                // and orphaning that plugin process). Acceptable
                // for now; a follow-up can claim the slot eagerly
                // via DashMap's entry API.
                let inner = self.inner.clone();
                let url = mcp.url;
                tokio::spawn(async move {
                    if let Ok(connection) =
                        inner.client.connect(url, None, None).await
                    {
                        // Build the per-connection state and install
                        // both list_changed pumps BEFORE storing.
                        // Callbacks read `interested_sessions` at fire
                        // time and fan one `McpListChanged` out per
                        // session via `fan_list_changed`. Empty set =
                        // silent no-op.
                        let state = Arc::new(PluginMcpState {
                            connection,
                            interested_sessions: dashmap::DashSet::new(),
                        });
                        let inner_t = inner.clone();
                        let state_t = state.clone();
                        state.connection.set_on_tools_list_changed(move || {
                            fan_list_changed(
                                &inner_t,
                                &state_t,
                                McpListChangedKind::Tools,
                            );
                        });
                        let inner_r = inner.clone();
                        let state_r = state.clone();
                        state.connection.set_on_resources_list_changed(move || {
                            fan_list_changed(
                                &inner_r,
                                &state_r,
                                McpListChangedKind::Resources,
                            );
                        });
                        inner.plugin_mcp_connections.insert(
                            (plugin_name, mcp_name),
                            state,
                        );
                    }
                });
                ok_response(id)
            }
            Ok(Err(message)) => error_response(id, 502, message),
            Err(_) => error_response(id, 504, "PLUGIN_MCP_CONNECT timed out"),
        }
    }
}

async fn forward_stderr(mut stderr: tokio::process::ChildStderr) {
    use tokio::io::AsyncReadExt;
    let mut buf = [0u8; 4096];
    loop {
        match stderr.read(&mut buf).await {
            Ok(0) | Err(_) => return,
            Ok(n) => {
                use std::io::Write;
                let _ = std::io::stderr().write_all(&buf[..n]);
            }
        }
    }
}

fn ok_response(id: String) -> server_response::Response {
    server_response::Response {
        id,
        status: 200,
        headers: IndexMap::new(),
        body: None,
    }
}

fn error_response(
    id: String,
    status: u16,
    message: impl Into<String>,
) -> server_response::Response {
    server_response::Response {
        id,
        status,
        headers: IndexMap::new(),
        body: Some(serde_json::json!({ "error": message.into() })),
    }
}

impl McpHandler for ConduitMcpHandler {
    async fn handle(&self, request: server_request::Request) -> server_response::Response {
        let id_for_err = request.id.clone();

        // Sentinel method: the API uses `PLUGIN_MCP_CONNECT` to ask
        // the CLI to start a plugin's MCP server, dial it locally,
        // and discard the connection. Dispatches before the
        // dial-to-upstream path because this RPC bypasses the
        // upstream MCP server entirely. The API only receives an
        // ack — no URL.
        if request.method.eq_ignore_ascii_case("PLUGIN_MCP_CONNECT") {
            return self.handle_plugin_mcp_connect(request).await;
        }

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

/// Fan a `list_changed` event from a plugin MCP connection out to
/// every interested ws_session_id via the WS notifier. The frame's
/// `mcp_session_id` is the PRIMARY upstream's session id for the
/// session (recorded during `initialize` handling) — that's the id
/// the API's GET-SSE handler uses to route the event to the proxy's
/// subscriber.
///
/// Drops events for sessions that haven't yet completed `initialize`
/// (no primary mcp_session_id recorded); the next `tools/list` for
/// that session will refresh state anyway. Drops the whole fan-out
/// if the WS notifier isn't installed yet.
fn fan_list_changed(
    inner: &Arc<Inner>,
    state: &Arc<PluginMcpState>,
    kind: McpListChangedKind,
) {
    let Some(notifier) = inner.notifier.get().cloned() else {
        return;
    };
    let interested: Vec<String> = state
        .interested_sessions
        .iter()
        .map(|s| s.clone())
        .collect();
    for ws_session_id in interested {
        let Some(sess) = inner.sessions.get(&ws_session_id) else {
            continue;
        };
        let Some(mcp_session_id) = sess.primary_mcp_session_id.get().cloned() else {
            continue;
        };
        let notifier = notifier.clone();
        tokio::spawn(async move {
            let _ = notifier
                .notify_list_changed(McpListChanged {
                    mcp_session_id,
                    kind,
                })
                .await;
        });
    }
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
        // Record the primary upstream's mcp_session_id on this
        // ws_session_id's `SessionState` — the `list_changed`
        // fan-out from selected plugin upstreams needs it as the
        // `mcp_session_id` field on every `McpListChanged` frame so
        // the API routes the event to the proxy's GET-SSE
        // subscriber correctly. `OnceLock::set` is first-write-wins
        // — harmless on resumes / repeated initializes.
        if let Some(ws_session_id) = ws_session_id_from_headers(&request.headers) {
            let sess = get_or_create_session(inner, &ws_session_id);
            let _ = sess
                .primary_mcp_session_id
                .set(state.connection.session_id.clone());
        }

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
            || k.eq_ignore_ascii_case(MCP_CONFIG_HEADER)
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

    // `tools/list`: apply the API↔CLI control surface stamped on
    // the request via `X-OBJECTIVEAI-MCP-CONFIG`.
    //
    // 1. Filter `result.tools[]` (from the primary upstream) by
    //    `names` + `objectiveai_builtins` — existing behavior.
    // 2. Aggregate the selected plugin MCP connections' tools into
    //    the same `result.tools[]`, prefix-namespaced by `mcp_name`.
    //    Reconcile `interested_sessions` against the diff between
    //    this session's previous selection and the current one.
    if rpc_method.as_deref() == Some("tools/list") {
        if let Some(config) = read_mcp_config_header(&request.headers) {
            if let Some(body) = resp_body.as_mut() {
                apply_tools_filter(inner, body, &config).await;
                let ws_session_id = ws_session_id_from_headers(&request.headers);
                aggregate_plugin_tools(
                    inner,
                    body,
                    &config.mcp_servers,
                    ws_session_id.as_deref(),
                )
                .await?;
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

/// Aggregate tools from the selected plugin MCP connections into the
/// primary upstream's `tools/list` response. Per the agent's
/// `client_objectiveai_mcp.plugins[i].mcp_servers` selection (carried
/// through `X-OBJECTIVEAI-MCP-CONFIG.mcp_servers`), look up each
/// matching `PluginMcpState`, call `Connection::list_tools` on it
/// concurrently, prefix each returned tool name `<mcp_name>_<tool>`
/// (mirrors `objectiveai-mcp-proxy/src/session.rs::prefix_name`'s
/// `<server>_<tool>` shape), and append to `result.tools[]`. If any
/// prefixed plugin tool collides with a primary tool's name, prefix
/// every primary tool with `objectiveai-mcp_` (the conventional
/// server-name for the local objectiveai-mcp upstream). Finally sort
/// the merged array by name for stable ordering.
///
/// Also reconciles `interested_sessions` on each `PluginMcpState`:
/// pairs added since the last `tools/list` for this ws_session_id
/// get this session added; removed pairs get it removed. Mutation
/// is gated by the session's `last_selected` mutex.
async fn aggregate_plugin_tools(
    inner: &Arc<Inner>,
    body: &mut serde_json::Value,
    selection: &[(String, String)],
    ws_session_id: Option<&str>,
) -> Result<(), ConduitError> {
    // Diff selection against this session's previous selection and
    // update interested_sessions accordingly.
    if let Some(ws_session_id) = ws_session_id {
        let sess = get_or_create_session(inner, ws_session_id);
        let mut last = sess.last_selected.lock().unwrap();
        let new_set: HashSet<(String, String)> = selection.iter().cloned().collect();
        let old_set: HashSet<(String, String)> = last.iter().cloned().collect();
        for removed in old_set.difference(&new_set) {
            if let Some(state) = inner.plugin_mcp_connections.get(removed) {
                state.interested_sessions.remove(ws_session_id);
            }
        }
        for added in new_set.difference(&old_set) {
            if let Some(state) = inner.plugin_mcp_connections.get(added) {
                state.interested_sessions.insert(ws_session_id.to_string());
            }
        }
        *last = selection.to_vec();
    }

    if selection.is_empty() {
        return Ok(());
    }

    // Fan out list_tools across selected plugin connections.
    let states: Vec<((String, String), Arc<PluginMcpState>)> = selection
        .iter()
        .filter_map(|pair| {
            inner
                .plugin_mcp_connections
                .get(pair)
                .map(|s| (pair.clone(), s.clone()))
        })
        .collect();
    let plugin_tool_lists: Vec<((String, String), Arc<Vec<objectiveai_sdk::mcp::tool::Tool>>)> =
        futures::future::try_join_all(states.into_iter().map(|(pair, state)| async move {
            let tools = state
                .connection
                .list_tools()
                .await
                .map_err(|_| ConduitError::PluginListTools)?;
            Ok::<_, ConduitError>((pair, tools))
        }))
        .await?;

    let Some(tools_arr) = body
        .get_mut("result")
        .and_then(|r| r.get_mut("tools"))
        .and_then(|t| t.as_array_mut())
    else {
        return Ok(());
    };

    // Build prefixed plugin tool entries.
    let mut plugin_entries: Vec<serde_json::Value> = Vec::new();
    for ((_plugin, mcp_name), arc) in plugin_tool_lists {
        for tool in arc.iter() {
            let prefixed_name = format!("{mcp_name}_{}", tool.name);
            let mut value = serde_json::to_value(tool).unwrap_or(serde_json::Value::Null);
            if let Some(obj) = value.as_object_mut() {
                obj.insert(
                    "name".to_string(),
                    serde_json::Value::String(prefixed_name),
                );
            }
            plugin_entries.push(value);
        }
    }

    // Conflict resolution: if any plugin tool name collides with a
    // primary tool name, prefix every primary tool with the
    // conventional `objectiveai-mcp_` namespace. Else primary stays
    // unprefixed.
    let plugin_names: HashSet<&str> = plugin_entries
        .iter()
        .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
        .collect();
    let primary_collides = tools_arr.iter().any(|t| {
        t.get("name")
            .and_then(|n| n.as_str())
            .map(|n| plugin_names.contains(n))
            .unwrap_or(false)
    });
    if primary_collides {
        for tool in tools_arr.iter_mut() {
            if let Some(obj) = tool.as_object_mut() {
                if let Some(name) = obj.get("name").and_then(|n| n.as_str()) {
                    let prefixed = format!("objectiveai-mcp_{name}");
                    obj.insert("name".to_string(), serde_json::Value::String(prefixed));
                }
            }
        }
    }

    // Append + sort by name.
    tools_arr.extend(plugin_entries);
    tools_arr.sort_by(|a, b| {
        let an = a.get("name").and_then(|n| n.as_str()).unwrap_or("");
        let bn = b.get("name").and_then(|n| n.as_str()).unwrap_or("");
        an.cmp(bn)
    });

    Ok(())
}

/// Decoded `X-OBJECTIVEAI-MCP-CONFIG` payload. The JSON control
/// surface the API stamps on every request to the synthetic
/// `/objectiveai-mcp` URL — drives both tools/list filtering AND
/// plugin MCP server selection.
#[derive(Debug, serde::Deserialize)]
struct McpConfig {
    /// Allow-listed primary-upstream tool names. See
    /// [`apply_tools_filter`].
    #[serde(default)]
    names: Vec<String>,
    /// Whether objectiveai-mcp built-ins pass the filter. See
    /// [`apply_tools_filter`].
    #[serde(default)]
    objectiveai_builtins: bool,
    /// `(plugin_name, mcp_name)` pairs the server has chosen as
    /// active for this ws_session_id. Drives `tools/list`
    /// aggregation across the primary upstream + selected plugin
    /// MCP connections and `list_changed` fan-out from selected
    /// plugin upstreams.
    #[serde(default)]
    mcp_servers: Vec<(String, String)>,
}

fn read_mcp_config_header(headers: &IndexMap<String, String>) -> Option<McpConfig> {
    use base64::Engine;
    let raw = headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(MCP_CONFIG_HEADER))?
        .1;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(raw.as_bytes())
        .ok()?;
    serde_json::from_slice::<McpConfig>(&bytes).ok()
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
    allowed: &McpConfig,
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

/// Extract the API↔CLI routing identifier (the `ws_session_id`) the
/// agent client stamps on every proxy-forwarded request.
fn ws_session_id_from_headers(headers: &IndexMap<String, String>) -> Option<String> {
    headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("X-OBJECTIVEAI-RESPONSE-ID"))
        .map(|(_, v)| v.clone())
}

/// Get-or-create the per-ws_session_id [`SessionState`]. Lazy: the
/// first request that carries an `X-OBJECTIVEAI-RESPONSE-ID` for a
/// given id materialises the entry; subsequent calls return the
/// same `Arc`.
fn get_or_create_session(inner: &Arc<Inner>, ws_session_id: &str) -> Arc<SessionState> {
    inner
        .sessions
        .entry(ws_session_id.to_string())
        .or_insert_with(|| {
            Arc::new(SessionState {
                last_selected: std::sync::Mutex::new(Vec::new()),
                primary_mcp_session_id: OnceLock::new(),
            })
        })
        .clone()
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
    #[error("plugin upstream list_tools failed")]
    PluginListTools,
}
