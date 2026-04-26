//! MCP Streamable-HTTP endpoint: parses inbound JSON-RPC, dispatches by method.

use axum::{
    extract::Json,
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use objectiveai::mcp::{
    JsonRpcError, JsonRpcRequest, JsonRpcResponse,
    initialize_result::{
        Implementation, InitializeResult, ServerCapabilities, ToolsCapability,
    },
};

/// MCP protocol version this proxy speaks.
const PROTOCOL_VERSION: &str = "2025-06-18";

/// JSON-RPC error code: method not found.
const METHOD_NOT_FOUND: i64 = -32601;

/// Mock session id used until real session management lands.
const MOCK_SESSION_ID: &str = "TODO";

/// Capabilities advertised to clients during `initialize`.
fn server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        experimental: None,
        logging: None,
        completions: None,
        prompts: None,
        resources: None,
        // Proxy's tool inventory is dynamic — tools come and go as upstream
        // MCP servers connect and disconnect.
        tools: Some(ToolsCapability {
            list_changed: Some(true),
        }),
        tasks: None,
    }
}

/// Server identity returned in `initialize`.
fn server_info() -> Implementation {
    Implementation {
        name: "objectiveai-mcp-proxy".into(),
        title: Some("ObjectiveAI MCP Proxy".into()),
        version: env!("CARGO_PKG_VERSION").into(),
        website_url: None,
        description: Some(
            "ObjectiveAI MCP proxy — multiplexes one client connection over many upstream MCP servers.".into(),
        ),
        icons: None,
    }
}

/// POST handler for the MCP Streamable HTTP endpoint.
pub async fn handle(Json(req): Json<serde_json::Value>) -> Response {
    // Notifications (no `id`) and requests share the same wire shape but
    // notifications return 202 Accepted with no body. Detect by absence of `id`.
    if req.get("id").is_none() {
        return StatusCode::ACCEPTED.into_response();
    }

    let request: JsonRpcRequest = match serde_json::from_value(req) {
        Ok(r) => r,
        Err(e) => return bad_request(format!("invalid JSON-RPC request: {e}")),
    };

    match request.method.as_str() {
        "initialize" => initialize(request),
        other => method_not_found(request.id, other),
    }
}

/// Construct the `initialize` response.
fn initialize(request: JsonRpcRequest) -> Response {
    let result = InitializeResult {
        protocol_version: PROTOCOL_VERSION.into(),
        capabilities: server_capabilities(),
        server_info: server_info(),
        instructions: Some(
            "ObjectiveAI MCP proxy. Multiplexes one client connection over many upstream MCP servers."
                .into(),
        ),
        _meta: None,
    };

    let body: JsonRpcResponse<InitializeResult> = JsonRpcResponse::Success {
        jsonrpc: "2.0".into(),
        id: request.id,
        result,
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        "Mcp-Session-Id",
        HeaderValue::from_static(MOCK_SESSION_ID),
    );

    (StatusCode::OK, headers, Json(body)).into_response()
}

/// Build a JSON-RPC `method not found` error response.
fn method_not_found(id: serde_json::Value, method: &str) -> Response {
    let body: JsonRpcResponse<()> = JsonRpcResponse::Error {
        jsonrpc: "2.0".into(),
        id,
        error: JsonRpcError {
            code: METHOD_NOT_FOUND,
            message: format!("method not found: {method}"),
            data: None,
        },
    };
    (StatusCode::OK, Json(body)).into_response()
}

/// Build a non-JSON-RPC bad-request response (invalid envelope, etc.).
fn bad_request(message: String) -> Response {
    (StatusCode::BAD_REQUEST, message).into_response()
}
