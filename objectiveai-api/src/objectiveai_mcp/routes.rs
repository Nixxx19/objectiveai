//! Axum sub-router + JSON-RPC dispatch for the API's MCP server.
//!
//! Three routes — exactly the MCP-spec surface (per
//! `objectiveai-mcp-proxy/src/run.rs`):
//!
//! - `POST /objectiveai-mcp`   — JSON-RPC dispatch on `method`
//! - `GET  /objectiveai-mcp`   — SSE notifications stream (delegated
//!   to the SDK conduit's [`handle_get_sse`])
//! - `DELETE /objectiveai-mcp` — session-terminate forward
//!
//! Routing key on every request: the `X-OBJECTIVEAI-RESPONSE-ID`
//! header. Missing → 400, unknown → 404. A single WS reverse-attach
//! can host many MCP sessions; `Mcp-Session-Id` (when present) rides
//! through to whatever upstream MCP server sits behind the CLI's
//! conduit and is opaque to this router.

use crate::objectiveai_mcp::{context::McpRequestContext, handlers};
use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use objectiveai_sdk::mcp::conduit::server::{
    McpListenerRegistry, ReverseChannelRegistry, handle_get_sse,
};

const JSON_RPC: &str = "2.0";
const RESPONSE_ID_HEADER: &str = "X-OBJECTIVEAI-RESPONSE-ID";

// ────────────────────────────────────────────────────────────────
// Router builder
// ────────────────────────────────────────────────────────────────

pub fn router(
    reverse_channels: ReverseChannelRegistry,
    listeners: McpListenerRegistry,
) -> axum::Router {
    let state = SharedState {
        reverse_channels,
        listeners,
    };
    axum::Router::new()
        .route(
            "/objectiveai-mcp",
            axum::routing::post(handle_post)
                .get(handle_get)
                .delete(handle_delete),
        )
        .with_state(state)
}

#[derive(Clone)]
struct SharedState {
    reverse_channels: ReverseChannelRegistry,
    listeners: McpListenerRegistry,
}

/// Pull the routing key out of the headers, verify a reverse channel
/// exists for it, and return the bare id. Returns the response to send
/// directly on the failure paths.
fn route(state: &SharedState, headers: &HeaderMap) -> Result<String, Response> {
    let response_id = headers
        .get(RESPONSE_ID_HEADER)
        .and_then(|v| v.to_str().ok())
        .map(str::to_string);
    let Some(response_id) = response_id else {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("missing {RESPONSE_ID_HEADER} header"),
        )
            .into_response());
    };
    if state.reverse_channels.get(&response_id).is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            format!("unknown response_id {response_id:?}"),
        )
            .into_response());
    }
    Ok(response_id)
}

fn build_ctx(
    state: &SharedState,
    response_id: String,
    headers: HeaderMap,
) -> McpRequestContext {
    McpRequestContext {
        response_id,
        headers,
        registry: state.reverse_channels.clone(),
    }
}

// ────────────────────────────────────────────────────────────────
// POST /objectiveai-mcp — JSON-RPC dispatch
// ────────────────────────────────────────────────────────────────

async fn handle_post(
    State(state): State<SharedState>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let response_id = match route(&state, &headers) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let envelope: serde_json::Value = match serde_json::from_slice(&body) {
        Ok(v) => v,
        Err(e) => return parse_error_response(format!("invalid JSON: {e}")),
    };

    let id = envelope.get("id").cloned().unwrap_or(serde_json::Value::Null);
    let method = envelope
        .get("method")
        .and_then(|m| m.as_str())
        .unwrap_or_default()
        .to_string();
    let params = envelope
        .get("params")
        .cloned()
        .unwrap_or(serde_json::Value::Null);

    let ctx = build_ctx(&state, response_id, headers);

    let result = match method.as_str() {
        "initialize" => dispatch_initialize(ctx, params).await,
        "ping" => dispatch_ping(ctx).await,
        "tools/list" => dispatch_tools_list(ctx, params).await,
        "tools/call" => dispatch_tools_call(ctx, params).await,
        "resources/list" => dispatch_resources_list(ctx, params).await,
        "resources/read" => dispatch_resources_read(ctx, params).await,
        other => return method_not_found(id, other),
    };

    match result {
        Ok(value) => json_rpc_success(id, value),
        Err(e) => json_rpc_error(id, e),
    }
}

// ────────────────────────────────────────────────────────────────
// Per-method dispatchers — parse params, call delegate, re-erase
// result to serde_json::Value for the success branch.
// ────────────────────────────────────────────────────────────────

async fn dispatch_initialize(
    ctx: McpRequestContext,
    params: serde_json::Value,
) -> Result<serde_json::Value, handlers::McpError> {
    let params = serde_json::from_value(params)
        .map_err(|e| invalid_params(format!("initialize: {e}")))?;
    let result = handlers::handle_initialize(ctx, params).await?;
    Ok(serde_json::to_value(result).expect("InitializeResult serializes"))
}

async fn dispatch_ping(
    ctx: McpRequestContext,
) -> Result<serde_json::Value, handlers::McpError> {
    handlers::handle_ping(ctx).await?;
    Ok(serde_json::json!({}))
}

async fn dispatch_tools_list(
    ctx: McpRequestContext,
    params: serde_json::Value,
) -> Result<serde_json::Value, handlers::McpError> {
    let params = serde_json::from_value(params)
        .map_err(|e| invalid_params(format!("tools/list: {e}")))?;
    let result = handlers::handle_tools_list(ctx, params).await?;
    Ok(serde_json::to_value(result).expect("ListToolsResult serializes"))
}

async fn dispatch_tools_call(
    ctx: McpRequestContext,
    params: serde_json::Value,
) -> Result<serde_json::Value, handlers::McpError> {
    let params = serde_json::from_value(params)
        .map_err(|e| invalid_params(format!("tools/call: {e}")))?;
    let result = handlers::handle_tools_call(ctx, params).await?;
    Ok(serde_json::to_value(result).expect("CallToolResult serializes"))
}

async fn dispatch_resources_list(
    ctx: McpRequestContext,
    params: serde_json::Value,
) -> Result<serde_json::Value, handlers::McpError> {
    let params = serde_json::from_value(params)
        .map_err(|e| invalid_params(format!("resources/list: {e}")))?;
    let result = handlers::handle_resources_list(ctx, params).await?;
    Ok(serde_json::to_value(result).expect("ListResourcesResult serializes"))
}

async fn dispatch_resources_read(
    ctx: McpRequestContext,
    params: serde_json::Value,
) -> Result<serde_json::Value, handlers::McpError> {
    let params = serde_json::from_value(params)
        .map_err(|e| invalid_params(format!("resources/read: {e}")))?;
    let result = handlers::handle_resources_read(ctx, params).await?;
    Ok(serde_json::to_value(result).expect("ReadResourceResult serializes"))
}

// ────────────────────────────────────────────────────────────────
// GET /objectiveai-mcp — SSE notifications stream
// ────────────────────────────────────────────────────────────────

async fn handle_get(
    State(state): State<SharedState>,
    headers: HeaderMap,
) -> Response {
    let response_id = match route(&state, &headers) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    handle_get_sse(response_id, state.listeners.clone(), headers).await
}

// ────────────────────────────────────────────────────────────────
// DELETE /objectiveai-mcp
// ────────────────────────────────────────────────────────────────

async fn handle_delete(
    State(state): State<SharedState>,
    headers: HeaderMap,
) -> Response {
    let response_id = match route(&state, &headers) {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let ctx = build_ctx(&state, response_id, headers);
    match handlers::handle_session_terminate(ctx).await {
        Ok(()) => StatusCode::OK.into_response(),
        Err(e) => mcp_error_to_http(e),
    }
}

// ────────────────────────────────────────────────────────────────
// Envelope + error helpers
// ────────────────────────────────────────────────────────────────

fn json_rpc_success(id: serde_json::Value, result: serde_json::Value) -> Response {
    axum::Json(serde_json::json!({
        "jsonrpc": JSON_RPC,
        "id": id,
        "result": result,
    }))
    .into_response()
}

fn json_rpc_error(id: serde_json::Value, e: handlers::McpError) -> Response {
    let mut err = serde_json::json!({ "code": e.code, "message": e.message });
    if let Some(data) = e.data {
        err["data"] = data;
    }
    axum::Json(serde_json::json!({
        "jsonrpc": JSON_RPC,
        "id": id,
        "error": err,
    }))
    .into_response()
}

fn method_not_found(id: serde_json::Value, method: &str) -> Response {
    json_rpc_error(
        id,
        handlers::McpError {
            code: -32601,
            message: format!("method not found: {method}"),
            data: None,
        },
    )
}

fn invalid_params(message: String) -> handlers::McpError {
    handlers::McpError {
        code: -32602,
        message,
        data: None,
    }
}

fn parse_error_response(message: String) -> Response {
    (
        StatusCode::BAD_REQUEST,
        axum::Json(serde_json::json!({
            "jsonrpc": JSON_RPC,
            "id": serde_json::Value::Null,
            "error": { "code": -32700, "message": message },
        })),
    )
        .into_response()
}

fn mcp_error_to_http(e: handlers::McpError) -> Response {
    let status = match e.code {
        -32601 => StatusCode::NOT_FOUND,
        -32602 => StatusCode::BAD_REQUEST,
        -32001 => StatusCode::NOT_FOUND,
        -32002 => StatusCode::SERVICE_UNAVAILABLE,
        -32003 => StatusCode::GATEWAY_TIMEOUT,
        -32004 => StatusCode::BAD_GATEWAY,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (status, e.message).into_response()
}
