//! `ConduitMcpHandler` — the CLI's reverse-attach implementation.
//!
//! The CLI talks to a **remote** `objectiveai-mcp` server. The
//! address comes from config (`config.mcp().get_address()` /
//! `get_port()`, env-var overridable via `OBJECTIVEAI_MCP_ADDRESS` /
//! `OBJECTIVEAI_MCP_PORT`), parsed the same way
//! `objectiveai-cli/src/api/client.rs` parses the API endpoint.
//!
//! On the first inbound `server_request` the handler dials the
//! remote MCP via `objectiveai_sdk::mcp::Client::connect`, with
//! that request's headers forwarded as-is. The resulting
//! [`Connection`] is held in an Arc'd Mutex inside the handler and
//! reused for every subsequent forwarded request — so all calls
//! that share a handler instance share one MCP session.
//!
//! Dispatch:
//! - `initialize` → synthesize from `connection.initialize_result`
//!   (the SDK already handshook). Strip
//!   `{tools,resources}.listChanged` from advertised capabilities
//!   so the proxy never subscribes — the chain stays single-shot.
//! - notifications (no `id`) → 202 Accepted, no body, never round-trip.
//! - everything else → raw POST through `connection.http_client` +
//!   `connection.url` + `connection.session_id`. Response parsed by
//!   [`parse_json_or_sse`] (rmcp's `StreamableHttpService` may pick
//!   either shape).

use indexmap::IndexMap;
use objectiveai_sdk::client_objectiveai_mcp::{server_request, server_response};
use objectiveai_sdk::http::McpHandler;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

struct ConduitState {
    connection: objectiveai_sdk::mcp::Connection,
}

#[derive(Clone)]
pub struct ConduitMcpHandler {
    inner: Arc<Inner>,
}

struct Inner {
    /// Configured remote MCP URL (e.g. `https://mcp.example.com`).
    /// `None` ⇒ MCP isn't configured for this CLI invocation; every
    /// request 501s the same way [`objectiveai_sdk::http::RejectHandler`]
    /// would.
    mcp_url: Option<String>,
    client: objectiveai_sdk::mcp::Client,
    state: Mutex<Option<Arc<ConduitState>>>,
}

impl ConduitMcpHandler {
    /// Construct a handler that dials the given URL on first use.
    /// `None` makes every `handle()` call reject with 501.
    pub fn new(mcp_url: Option<String>) -> Self {
        let http = reqwest::Client::builder()
            .build()
            .expect("reqwest::Client::build is infallible without rustls toggles");
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
        Self {
            inner: Arc::new(Inner {
                mcp_url,
                client,
                state: Mutex::new(None),
            }),
        }
    }
}

impl McpHandler for ConduitMcpHandler {
    async fn handle(&self, request: server_request::Request) -> server_response::Response {
        let id_for_err = request.id.clone();

        let Some(mcp_url) = self.inner.mcp_url.as_ref() else {
            return reject_no_mcp(id_for_err);
        };

        // Lazy connect on first call.
        let state = {
            let mut guard = self.inner.state.lock().await;
            match guard.as_ref() {
                Some(s) => s.clone(),
                None => {
                    let mut connect_headers = request.headers.clone();
                    // Hop-by-hop and layer-internal: don't propagate to MCP.
                    for k in ["Host", "host", "Content-Length", "content-length",
                              "Mcp-Session-Id", "mcp-session-id"] {
                        connect_headers.shift_remove(k);
                    }
                    let connection = match self
                        .inner
                        .client
                        .connect(mcp_url.clone(), None, Some(connect_headers))
                        .await
                    {
                        Ok(c) => c,
                        Err(e) => return conduit_error(id_for_err, format!("connect: {e}")),
                    };
                    let st = Arc::new(ConduitState { connection });
                    *guard = Some(st.clone());
                    st
                }
            }
        };

        match forward(&state, request).await {
            Ok(resp) => resp,
            Err(e) => conduit_error(id_for_err, e.to_string()),
        }
    }
}

async fn forward(
    state: &ConduitState,
    request: server_request::Request,
) -> Result<server_response::Response, ConduitError> {
    let envelope = request.body.clone();

    // Notifications (no `id`) → 202 with no body; don't round-trip.
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

    // `initialize`: synthesize from the SDK Connection's cached
    // InitializeResult; don't re-handshake.
    if rpc_method.as_deref() == Some("initialize") {
        let mut init_value = serde_json::to_value(&state.connection.initialize_result)
            .map_err(ConduitError::Serialize)?;
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
            // Local-layer session id + the content-type the API
            // will re-set verbatim. Stripped.
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
                "message": "this client has no MCP server configured (set `objectiveai mcp address`)",
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
/// reparsing for SSE-wrapped responses. Mirrors
/// `objectiveai_sdk::mcp::transport::parse_streamable_http_response`.
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

/// Build a handler for the current process by reading the MCP
/// address out of the on-disk config (with env-var overrides).
/// Mirrors `crate::api::client::build_http_client`'s config-loading
/// pattern. Returns a handler that rejects every request with 501
/// when no MCP address is configured.
pub fn build_handler(
    config: &mut objectiveai_sdk::filesystem::config::Config,
) -> ConduitMcpHandler {
    let mcp_url = std::env::var("OBJECTIVEAI_MCP_ADDRESS").ok().or_else(|| {
        let mcp = config.mcp();
        let port = std::env::var("OBJECTIVEAI_MCP_PORT")
            .ok()
            .and_then(|s| s.parse::<u16>().ok())
            .or_else(|| mcp.get_port());
        crate::api::client::compose_url(mcp.get_address(), port)
    });
    ConduitMcpHandler::new(mcp_url)
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
