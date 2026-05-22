//! Streamable-HTTP MCP endpoint the API hosts on
//! `/objectiveai-mcp/{session_id}`. Acts as the local MCP server an
//! `objectiveai-mcp-proxy` upstream dials for objectiveai-managed
//! tools (the built-in `ObjectiveAI` CLI catch-all plus any plugins
//! / tools the calling client exposes).
//!
//! Endpoint state is a [`crate::streaming_ws::ReverseChannelRegistry`]
//! — a process-wide map of `session_id` -> live WS `ReverseChannel`.
//! Every JSON-RPC request that needs the calling client (today
//! `tools/list` and `tools/call`) is forwarded as a
//! [`server_request::Request`](objectiveai_sdk::client_objectiveai_mcp::server_request::Request)
//! frame on that WS, and the matching
//! [`server_response::Response`](objectiveai_sdk::client_objectiveai_mcp::server_response::Response)
//! is translated back into a JSON-RPC reply.
//!
//! `initialize` / `notifications/initialized` / `ping` are answered
//! locally and never touch the reverse channel — they're protocol
//! handshake that doesn't depend on the client's MCP surface.

use std::time::Duration;

use axum::{
    body::Bytes,
    http::{HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use objectiveai_sdk::client_objectiveai_mcp::{
    server_request::Payload as ServerRequestPayload,
    server_response::Result as ServerResponseResult,
};
use objectiveai_sdk::mcp::{
    JsonRpcError, JsonRpcRequest, JsonRpcResponse,
    initialize_result::{
        Implementation, InitializeResult, ServerCapabilities, ToolsCapability,
    },
    tool::{CallToolRequestParams, ListToolsRequest},
};

use crate::streaming_ws::{
    ReverseChannel, ReverseChannelRegistry, send_server_request,
};

/// MCP protocol version we advertise. Same value the proxy
/// pins (`objectiveai-mcp-proxy/src/mcp.rs::PROTOCOL_VERSION`).
const PROTOCOL_VERSION: &str = "2025-06-18";

/// MCP `Mcp-Session-Id` request/response header.
const SESSION_ID_HEADER: &str = "Mcp-Session-Id";

/// JSON-RPC error codes.
const PARSE_ERROR: i64 = -32700;
const INVALID_REQUEST: i64 = -32600;
const METHOD_NOT_FOUND: i64 = -32601;
const INVALID_PARAMS: i64 = -32602;
const INTERNAL_ERROR: i64 = -32603;

/// How long to wait for a `server_response` before failing with
/// `INTERNAL_ERROR`. Matches the proxy's default call timeout
/// (`objectiveai-api/src/run.rs::375` ish).
const REVERSE_CHANNEL_TIMEOUT: Duration = Duration::from_secs(30);

/// `POST /objectiveai-mcp/{session_id}` — accept one JSON-RPC envelope
/// and dispatch by method.
pub async fn handle_post(
    session_id: String,
    registry: ReverseChannelRegistry,
    body: Bytes,
) -> Response {
    let body: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => return parse_error_response(format!("invalid JSON: {e}")),
    };

    // Notifications (no `id`) get 202 Accepted with no body.
    if body.get("id").is_none() {
        return StatusCode::ACCEPTED.into_response();
    }

    let request: JsonRpcRequest = match serde_json::from_value(body) {
        Ok(r) => r,
        Err(e) => return parse_error_response(format!("invalid JSON-RPC envelope: {e}")),
    };

    match request.method.as_str() {
        "initialize" => handle_initialize(&session_id, request),
        "ping" => handle_ping(request),
        "tools/list" => handle_tools_list(&registry, &session_id, request).await,
        "tools/call" => handle_tools_call(&registry, &session_id, request).await,
        other => method_not_found(request.id, other),
    }
}

/// `DELETE /objectiveai-mcp/{session_id}` — explicit session
/// termination. The reverse channel registry is owned by the WS
/// handler lifecycle, so we don't remove anything here; just return
/// 200 so clients that issue DELETE on disconnect don't error.
pub async fn handle_delete() -> Response {
    StatusCode::OK.into_response()
}

/// `GET /objectiveai-mcp/{session_id}` — server-initiated SSE
/// stream. We don't push notifications from this endpoint (the agent
/// completion's send loop carries everything the client needs), so
/// we 405 to make that explicit.
pub async fn handle_get() -> Response {
    (StatusCode::METHOD_NOT_ALLOWED, "GET not supported").into_response()
}

fn handle_initialize(session_id: &str, request: JsonRpcRequest) -> Response {
    let result = InitializeResult {
        protocol_version: PROTOCOL_VERSION.to_string(),
        capabilities: ServerCapabilities {
            experimental: None,
            logging: None,
            completions: None,
            prompts: None,
            resources: None,
            tools: Some(ToolsCapability {
                list_changed: None,
            }),
            tasks: None,
        },
        server_info: Implementation {
            name: "objectiveai".into(),
            title: None,
            version: env!("CARGO_PKG_VERSION").into(),
            website_url: None,
            description: None,
            icons: None,
        },
        instructions: None,
        _meta: None,
    };
    let mut resp = json_rpc_success(request.id, &result);
    if let Ok(value) = HeaderValue::from_str(session_id) {
        resp.headers_mut().insert(SESSION_ID_HEADER, value);
    }
    resp
}

fn handle_ping(request: JsonRpcRequest) -> Response {
    json_rpc_success(request.id, &serde_json::Map::<String, _>::new())
}

async fn handle_tools_list(
    registry: &ReverseChannelRegistry,
    session_id: &str,
    request: JsonRpcRequest,
) -> Response {
    let rc = match registry.get(session_id) {
        Some(rc) => rc.clone(),
        None => return session_not_found(request.id, session_id),
    };
    let params: ListToolsRequest = match request.params.clone() {
        Some(v) => match serde_json::from_value(v) {
            Ok(p) => p,
            Err(e) => {
                return json_rpc_error(
                    request.id,
                    INVALID_PARAMS,
                    &format!("tools/list params: {e}"),
                );
            }
        },
        None => ListToolsRequest { cursor: None },
    };
    forward(&rc, request.id, ServerRequestPayload::McpToolsList(params)).await
}

async fn handle_tools_call(
    registry: &ReverseChannelRegistry,
    session_id: &str,
    request: JsonRpcRequest,
) -> Response {
    let rc = match registry.get(session_id) {
        Some(rc) => rc.clone(),
        None => return session_not_found(request.id, session_id),
    };
    let params: CallToolRequestParams = match request.params.clone() {
        Some(v) => match serde_json::from_value(v) {
            Ok(p) => p,
            Err(e) => {
                return json_rpc_error(
                    request.id,
                    INVALID_PARAMS,
                    &format!("tools/call params: {e}"),
                );
            }
        },
        None => {
            return json_rpc_error(
                request.id,
                INVALID_PARAMS,
                "tools/call requires params",
            );
        }
    };
    forward(&rc, request.id, ServerRequestPayload::McpToolsCall(params)).await
}

/// Issue a `server_request` over the reverse channel and translate
/// the matching `server_response` into a JSON-RPC reply.
async fn forward(
    rc: &ReverseChannel,
    rpc_id: serde_json::Value,
    payload: ServerRequestPayload,
) -> Response {
    let rx = match send_server_request(&rc.sink, &rc.pending, payload).await {
        Ok(rx) => rx,
        Err(()) => {
            return json_rpc_error(
                rpc_id,
                INTERNAL_ERROR,
                "reverse channel closed before request could be sent",
            );
        }
    };
    let result = match tokio::time::timeout(REVERSE_CHANNEL_TIMEOUT, rx).await {
        Ok(Ok(r)) => r,
        Ok(Err(_)) => {
            return json_rpc_error(
                rpc_id,
                INTERNAL_ERROR,
                "reverse channel dropped before response arrived",
            );
        }
        Err(_) => {
            return json_rpc_error(
                rpc_id,
                INTERNAL_ERROR,
                "reverse channel timed out waiting for response",
            );
        }
    };
    match result {
        ServerResponseResult::Ok { value } => json_rpc_success_value(rpc_id, value),
        ServerResponseResult::Error { code, message } => json_rpc_error_value(
            rpc_id,
            INTERNAL_ERROR,
            format!("reverse channel returned error code={code} message={message}"),
        ),
    }
}

// ----- JSON-RPC response builders -------------------------------------------

fn json_rpc_success<T: serde::Serialize>(id: serde_json::Value, result: &T) -> Response {
    let value = match serde_json::to_value(result) {
        Ok(v) => v,
        Err(e) => return json_rpc_error(id, INTERNAL_ERROR, &format!("serialize: {e}")),
    };
    json_rpc_success_value(id, value)
}

fn json_rpc_success_value(id: serde_json::Value, value: serde_json::Value) -> Response {
    let env = JsonRpcResponse::<serde_json::Value>::Success {
        jsonrpc: "2.0".to_string(),
        id,
        result: value,
    };
    axum::Json(env).into_response()
}

fn json_rpc_error(id: serde_json::Value, code: i64, message: &str) -> Response {
    json_rpc_error_value(id, code, message.to_string())
}

fn json_rpc_error_value(id: serde_json::Value, code: i64, message: String) -> Response {
    let env = JsonRpcResponse::<serde_json::Value>::Error {
        jsonrpc: "2.0".to_string(),
        id,
        error: JsonRpcError {
            code,
            message,
            data: None,
        },
    };
    axum::Json(env).into_response()
}

fn parse_error_response(message: String) -> Response {
    let env = JsonRpcResponse::<serde_json::Value>::Error {
        jsonrpc: "2.0".to_string(),
        id: serde_json::Value::Null,
        error: JsonRpcError {
            code: PARSE_ERROR,
            message,
            data: None,
        },
    };
    axum::Json(env).into_response()
}

fn method_not_found(id: serde_json::Value, method: &str) -> Response {
    let _ = INVALID_REQUEST; // keep symbol live; spec-defined but unused locally
    json_rpc_error(id, METHOD_NOT_FOUND, &format!("method not found: {method}"))
}

fn session_not_found(id: serde_json::Value, session_id: &str) -> Response {
    json_rpc_error(
        id,
        INTERNAL_ERROR,
        &format!("no reverse channel for session_id {session_id:?}"),
    )
}
