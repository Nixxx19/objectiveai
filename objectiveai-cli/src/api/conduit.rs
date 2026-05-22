//! `ConduitMcpHandler` — the CLI's reverse-attach implementation.
//!
//! When a streaming CLI command runs against an agent that declares
//! `client_objectiveai_mcp`, the API forwards every MCP HTTP request
//! the proxy makes over the per-WS reverse channel. This handler
//! receives those forwarded requests, spawns `objectiveai-mcp` as a
//! local subprocess on the first call, and pipes subsequent requests
//! through to it. Replies come back as plain JSON-RPC; we never
//! emit SSE.
//!
//! Lifecycle: the handler is bound at `create_*_streaming` time and
//! dropped when the stream ends. `tokio::process::Child` is held
//! with `kill_on_drop(true)`, so the subprocess dies cleanly when
//! the stream finishes or the CLI is interrupted.

use indexmap::IndexMap;
use objectiveai_sdk::client_objectiveai_mcp::{server_request, server_response};
use objectiveai_sdk::http::McpHandler;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Lazily-spawned state. None until the first `handle()` call wins
/// the spawn race; after that, every call reuses the same
/// subprocess + reqwest client.
struct ConduitState {
    /// Drops + SIGKILLs the subprocess when this state is dropped.
    _child: tokio::process::Child,
    /// `http://127.0.0.1:<port>` — the subprocess's listener.
    base_url: String,
    http: reqwest::Client,
}

pub struct ConduitMcpHandler {
    inner: Arc<Mutex<Option<Arc<ConduitState>>>>,
}

impl Default for ConduitMcpHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ConduitMcpHandler {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(None)),
        }
    }

    /// Spawn (once) and return the live state. Subsequent calls hit
    /// the fast path that just clones the existing `Arc`.
    async fn ensure_spawned(&self) -> Result<Arc<ConduitState>, ConduitError> {
        let mut guard = self.inner.lock().await;
        if let Some(state) = guard.as_ref() {
            return Ok(state.clone());
        }

        // Grab a free port by binding ephemerally, reading the
        // assigned port, then dropping the listener. There is a
        // tiny race where another process could grab the same port
        // between drop and respawn, but in practice this is fine
        // for ephemeral local subprocesses.
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
            // Quiet the subprocess unless someone overrides via env.
            .env("RUST_LOG", std::env::var("OBJECTIVEAI_MCP_LOG").unwrap_or_else(|_| "warn".to_string()))
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::inherit())
            .kill_on_drop(true)
            .spawn()
            .map_err(ConduitError::Spawn)?;

        // Poll the port until the subprocess accepts a TCP
        // connection. Bounded to a few seconds; we'd rather fail
        // fast and surface an error than hang the WS forever.
        let base_url = format!("http://127.0.0.1:{port}");
        let ready_deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            if std::time::Instant::now() > ready_deadline {
                return Err(ConduitError::ReadyTimeout);
            }
            if tokio::net::TcpStream::connect(("127.0.0.1", port)).await.is_ok() {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        }

        let http = reqwest::Client::builder()
            .build()
            .map_err(ConduitError::ClientBuild)?;

        let state = Arc::new(ConduitState {
            _child: child,
            base_url,
            http,
        });
        *guard = Some(state.clone());
        Ok(state)
    }

    async fn forward(
        &self,
        request: server_request::Request,
    ) -> Result<server_response::Response, ConduitError> {
        let state = self.ensure_spawned().await?;

        let method = reqwest::Method::from_bytes(request.method.as_bytes())
            .map_err(|_| ConduitError::BadMethod(request.method.clone()))?;

        let mut req = state.http.request(method, &state.base_url);

        // Pass through every header except hop-by-hop and ones we'd
        // be overriding ourselves (`host`, `content-length`,
        // `accept`).
        for (k, v) in &request.headers {
            if k.eq_ignore_ascii_case("host")
                || k.eq_ignore_ascii_case("content-length")
                || k.eq_ignore_ascii_case("accept")
                || k.eq_ignore_ascii_case("connection")
            {
                continue;
            }
            req = req.header(k, v);
        }
        // Force JSON-only — the calling client doesn't want SSE on
        // this leg, and the local objectiveai-mcp will happily
        // serve JSON when asked.
        req = req.header("Accept", "application/json");

        if let Some(body) = &request.body {
            req = req.json(body);
        }

        let resp = req.send().await.map_err(ConduitError::Request)?;
        let status = resp.status().as_u16();
        let mut resp_headers: IndexMap<String, String> = IndexMap::new();
        for (k, v) in resp.headers().iter() {
            if let Ok(value) = v.to_str() {
                resp_headers.insert(k.as_str().to_string(), value.to_string());
            }
        }
        let resp_text = resp.text().await.map_err(ConduitError::Body)?;
        let resp_body: Option<serde_json::Value> = if resp_text.is_empty() {
            None
        } else {
            serde_json::from_str(&resp_text).ok()
        };

        // Strip `tools.listChanged` from any initialize response so
        // the proxy never subscribes to notifications. Keeps the
        // chain single-shot end-to-end.
        let resp_body = strip_list_changed(resp_body);

        Ok(server_response::Response {
            id: request.id,
            status,
            headers: resp_headers,
            body: resp_body,
        })
    }
}

impl McpHandler for ConduitMcpHandler {
    async fn handle(&self, request: server_request::Request) -> server_response::Response {
        let id = request.id.clone();
        match self.forward(request).await {
            Ok(r) => r,
            Err(e) => server_response::Response {
                id,
                status: 502,
                headers: IndexMap::new(),
                body: Some(serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": serde_json::Value::Null,
                    "error": {
                        "code": -32603,
                        "message": format!("conduit: {e}"),
                    },
                })),
            },
        }
    }
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
    #[error("invalid HTTP method {0:?}")]
    BadMethod(String),
    #[error("forwarding HTTP request failed: {0}")]
    Request(reqwest::Error),
    #[error("reading response body failed: {0}")]
    Body(reqwest::Error),
}

/// Walks a JSON-RPC `initialize` response and removes
/// `result.capabilities.tools.listChanged` (and the sibling
/// `result.capabilities.resources.listChanged`) so the proxy doesn't
/// open an SSE subscription it can't traverse the reverse-attach
/// anyway. No-op on every other response shape.
fn strip_list_changed(body: Option<serde_json::Value>) -> Option<serde_json::Value> {
    let mut body = body?;
    if let Some(caps) = body.pointer_mut("/result/capabilities") {
        if let Some(obj) = caps.as_object_mut() {
            if let Some(tools) = obj.get_mut("tools").and_then(|t| t.as_object_mut()) {
                tools.remove("listChanged");
            }
            if let Some(resources) = obj.get_mut("resources").and_then(|r| r.as_object_mut()) {
                resources.remove("listChanged");
            }
        }
    }
    Some(body)
}
