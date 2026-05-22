//! `ConduitMcpHandler` — the CLI's reverse-attach implementation.
//!
//! Spawns `objectiveai-mcp` as a subprocess on first use and holds a
//! [`objectiveai_sdk::mcp::Connection`] to it for the CLI's
//! lifetime. Every inbound `server_request` from the API is
//! translated into one of two paths:
//!
//! - `initialize` — answered locally from the connection's cached
//!   [`InitializeResult`]; we don't re-handshake. We also strip
//!   `{tools,resources}.listChanged` from the advertised capabilities
//!   so the proxy never subscribes to notifications (the chain is
//!   single-shot end-to-end).
//! - everything else — forwarded as a raw HTTP POST through the
//!   Connection's `http_client` + `url` + `session_id`. Response is
//!   parsed as bare JSON or SSE-wrapped JSON (rmcp's
//!   `StreamableHttpService` may pick either; the SDK already
//!   tolerates both).
//!
//! The Connection is created with the headers from the **first**
//! forwarded request — typically the proxy's `initialize` POST.
//! Subsequent requests are dispatched against that same Connection;
//! their headers don't influence connect parameters.

use indexmap::IndexMap;
use objectiveai_sdk::client_objectiveai_mcp::{server_request, server_response};
use objectiveai_sdk::http::McpHandler;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::OnceCell;

/// Process-wide singleton holding the subprocess + SDK Connection.
/// Initialized lazily on the first `ConduitMcpHandler::handle` call
/// (any `ConduitMcpHandler` instance shares this). Lives for the
/// CLI process lifetime; the subprocess is killed by the OS when
/// the CLI exits (kill_on_drop fires if the static is ever
/// dropped, which Rust's runtime doesn't guarantee — orphaned
/// children get reparented to init).
static STATE: OnceCell<Arc<ConduitState>> = OnceCell::const_new();

struct ConduitState {
    _child: tokio::process::Child,
    connection: objectiveai_sdk::mcp::Connection,
}

#[derive(Default)]
pub struct ConduitMcpHandler;

impl ConduitMcpHandler {
    pub fn new() -> Self {
        Self
    }
}

impl McpHandler for ConduitMcpHandler {
    async fn handle(&self, request: server_request::Request) -> server_response::Response {
        let id_for_err = request.id.clone();
        let state = match STATE
            .get_or_try_init(|| ensure_state(request.headers.clone()))
            .await
        {
            Ok(s) => s.clone(),
            Err(e) => return conduit_error(id_for_err, format!("init: {e}")),
        };

        match forward(&state, request).await {
            Ok(resp) => resp,
            Err(e) => conduit_error(id_for_err, e.to_string()),
        }
    }
}

/// Spawn objectiveai-mcp, dial it via the SDK Client, return the
/// live state. Only ever called once (by `OnceCell`).
async fn ensure_state(
    first_request_headers: IndexMap<String, String>,
) -> Result<Arc<ConduitState>, ConduitError> {
    // Grab a free port by binding ephemerally.
    let listener = std::net::TcpListener::bind("127.0.0.1:0")
        .map_err(ConduitError::PortBind)?;
    let port = listener
        .local_addr()
        .map_err(ConduitError::PortBind)?
        .port();
    drop(listener);

    let child = tokio::process::Command::new("objectiveai-mcp")
        .env("ADDRESS", "127.0.0.1")
        .env("PORT", port.to_string())
        .env(
            "RUST_LOG",
            std::env::var("OBJECTIVEAI_MCP_LOG").unwrap_or_else(|_| "warn".to_string()),
        )
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::inherit())
        .kill_on_drop(true)
        .spawn()
        .map_err(ConduitError::Spawn)?;

    // Wait for the subprocess to accept connections.
    let url = format!("http://127.0.0.1:{port}");
    let ready_deadline = std::time::Instant::now() + Duration::from_secs(5);
    loop {
        if std::time::Instant::now() > ready_deadline {
            return Err(ConduitError::ReadyTimeout);
        }
        if tokio::net::TcpStream::connect(("127.0.0.1", port))
            .await
            .is_ok()
        {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    // Build the SDK MCP Client + connect with the proxy's headers.
    // Mcp-Session-Id is stripped before forwarding — the proxy's
    // session id belongs to a different layer; rmcp will mint its
    // own and the Connection tracks it internally.
    let http = reqwest::Client::builder()
        .build()
        .map_err(ConduitError::ClientBuild)?;
    let client = objectiveai_sdk::mcp::Client::new(
        http,
        "objectiveai-cli-conduit".to_string(),
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
    let mut connect_headers = first_request_headers;
    connect_headers.shift_remove("Mcp-Session-Id");
    connect_headers.shift_remove("mcp-session-id");
    connect_headers.shift_remove("Host");
    connect_headers.shift_remove("host");
    connect_headers.shift_remove("Content-Length");
    connect_headers.shift_remove("content-length");
    let connection = client
        .connect(url, None, Some(connect_headers))
        .await
        .map_err(|e| ConduitError::Connect(e.to_string()))?;

    Ok(Arc::new(ConduitState {
        _child: child,
        connection,
    }))
}

/// Dispatch one inbound `server_request` against the live state and
/// build a `server_response`.
async fn forward(
    state: &ConduitState,
    request: server_request::Request,
) -> Result<server_response::Response, ConduitError> {
    let envelope = request.body.clone();

    // Notifications (no `id`) and bodyless methods get 202.
    let rpc_id = envelope
        .as_ref()
        .and_then(|v| v.get("id"))
        .cloned();
    let rpc_method = envelope
        .as_ref()
        .and_then(|v| v.get("method"))
        .and_then(|m| m.as_str())
        .map(|s| s.to_string());
    if rpc_id.is_none() {
        return Ok(server_response::Response {
            id: request.id,
            status: 202,
            headers: IndexMap::new(),
            body: None,
        });
    }

    // `initialize`: synthesize from cached InitializeResult; don't
    // re-handshake (the Connection already did it).
    if rpc_method.as_deref() == Some("initialize") {
        let mut init_value = serde_json::to_value(&state.connection.initialize_result)
            .map_err(ConduitError::Serialize)?;
        // Strip listChanged advertisements so the proxy never
        // subscribes — keeps the chain single-shot.
        if let Some(caps) = init_value.pointer_mut("/capabilities") {
            if let Some(obj) = caps.as_object_mut() {
                if let Some(tools) = obj.get_mut("tools").and_then(|t| t.as_object_mut()) {
                    tools.remove("listChanged");
                }
                if let Some(resources) =
                    obj.get_mut("resources").and_then(|r| r.as_object_mut())
                {
                    resources.remove("listChanged");
                }
            }
        }
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": rpc_id.unwrap(),
            "result": init_value,
        });
        return Ok(server_response::Response {
            id: request.id,
            status: 200,
            headers: IndexMap::new(),
            body: Some(body),
        });
    }

    // Everything else: forward as a raw POST through the
    // Connection's http_client + url + session_id.
    let conn = &state.connection;
    let mut req = conn.http_client.post(&conn.url);
    for (k, v) in &request.headers {
        if k.eq_ignore_ascii_case("host")
            || k.eq_ignore_ascii_case("content-length")
            || k.eq_ignore_ascii_case("connection")
            || k.eq_ignore_ascii_case("accept")
            || k.eq_ignore_ascii_case("content-type")
            || k.eq_ignore_ascii_case("mcp-session-id")
        {
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
        {
            // Stripped — these belong to the local layer or are
            // about to be re-set by the API.
            continue;
        }
        if let Ok(value) = v.to_str() {
            resp_headers.insert(k.as_str().to_string(), value.to_string());
        }
    }
    let resp_text = resp.text().await.map_err(ConduitError::Body)?;
    let resp_body = parse_json_or_sse(&resp_text);

    Ok(server_response::Response {
        id: request.id,
        status,
        headers: resp_headers,
        body: resp_body,
    })
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

/// Try bare JSON first, then strip `data:` prefixes from each line
/// and reparse. Mirrors `objectiveai_sdk::mcp::transport::parse_streamable_http_response`.
/// Empty body → `None`.
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
    #[error("could not bind a local port: {0}")]
    PortBind(std::io::Error),
    #[error("could not spawn objectiveai-mcp subprocess: {0}")]
    Spawn(std::io::Error),
    #[error("objectiveai-mcp subprocess did not become ready in time")]
    ReadyTimeout,
    #[error("could not build reqwest client: {0}")]
    ClientBuild(reqwest::Error),
    #[error("MCP connect failed: {0}")]
    Connect(String),
    #[error("forwarding HTTP request failed: {0}")]
    Request(reqwest::Error),
    #[error("reading response body failed: {0}")]
    Body(reqwest::Error),
    #[error("serializing InitializeResult failed: {0}")]
    Serialize(serde_json::Error),
}
