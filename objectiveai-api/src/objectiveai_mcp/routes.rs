//! Axum sub-router + JSON-RPC dispatch for the API's MCP server.
//! Real dispatch code; every leaf delegate it calls is `todo!()`
//! (see [`super::handlers`]).

use crate::objectiveai_mcp::{context::McpRequestContext, handlers};
use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use objectiveai_sdk::mcp::conduit::server::{
    McpListenerRegistry, ReverseChannelRegistry, handle_get_sse,
};

const JSON_RPC: &str = "2.0";

// ────────────────────────────────────────────────────────────────
// Router builder
// ────────────────────────────────────────────────────────────────

/// Build the axum sub-router for `/objectiveai-mcp/{session_id}` and
/// its `/notify` children. Six routes total — see plan inventory.
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
            "/objectiveai-mcp/{session_id}",
            axum::routing::post(handle_post)
                .get(handle_get)
                .delete(handle_delete),
        )
        .route(
            "/objectiveai-mcp/{session_id}/notify",
            axum::routing::post(handle_notify_post).get(handle_notify_get),
        )
        .route(
            "/objectiveai-mcp/{session_id}/notify/queued",
            axum::routing::get(handle_notify_queued_get),
        )
        .with_state(state)
}

#[derive(Clone)]
struct SharedState {
    reverse_channels: ReverseChannelRegistry,
    listeners: McpListenerRegistry,
}

fn build_ctx(state: &SharedState, session_id: String, headers: HeaderMap) -> McpRequestContext {
    McpRequestContext {
        session_id,
        headers,
        registry: state.reverse_channels.clone(),
        listeners: state.listeners.clone(),
    }
}

// ────────────────────────────────────────────────────────────────
// POST /objectiveai-mcp/{session_id} — JSON-RPC dispatch
// ────────────────────────────────────────────────────────────────

async fn handle_post(
    State(state): State<SharedState>,
    Path(session_id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    // 404 fast if the WS hasn't reverse-attached for this session.
    if state.reverse_channels.get(&session_id).is_none() {
        return (StatusCode::NOT_FOUND, "unknown session").into_response();
    }

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

    let ctx = build_ctx(&state, session_id, headers);

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
// GET /objectiveai-mcp/{session_id} — SSE notifications stream
// ────────────────────────────────────────────────────────────────

async fn handle_get(
    State(state): State<SharedState>,
    Path(session_id): Path<String>,
    headers: HeaderMap,
) -> Response {
    handle_get_sse(session_id, state.listeners.clone(), headers).await
}

// ────────────────────────────────────────────────────────────────
// DELETE /objectiveai-mcp/{session_id}
// ────────────────────────────────────────────────────────────────

async fn handle_delete(
    State(state): State<SharedState>,
    Path(session_id): Path<String>,
    headers: HeaderMap,
) -> Response {
    let ctx = build_ctx(&state, session_id, headers);
    match handlers::handle_session_terminate(ctx).await {
        Ok(()) => StatusCode::OK.into_response(),
        Err(e) => mcp_error_to_http(e),
    }
}

// ────────────────────────────────────────────────────────────────
// /notify routes
// ────────────────────────────────────────────────────────────────

async fn handle_notify_post(
    State(state): State<SharedState>,
    Path(session_id): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let blocks = match serde_json::from_slice(&body) {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                format!("invalid /notify body: {e}"),
            )
                .into_response();
        }
    };
    let ctx = build_ctx(&state, session_id, headers);
    match handlers::handle_notify_enqueue(ctx, blocks).await {
        Ok(()) => StatusCode::ACCEPTED.into_response(),
        Err(e) => mcp_error_to_http(e),
    }
}

async fn handle_notify_get(
    State(state): State<SharedState>,
    Path(session_id): Path<String>,
    headers: HeaderMap,
) -> Response {
    let ctx = build_ctx(&state, session_id, headers);
    match handlers::handle_notify_drain(ctx).await {
        Ok(blocks) => (StatusCode::OK, axum::Json(blocks)).into_response(),
        Err(e) => mcp_error_to_http(e),
    }
}

async fn handle_notify_queued_get(
    State(state): State<SharedState>,
    Path(session_id): Path<String>,
    headers: HeaderMap,
) -> Response {
    let ctx = build_ctx(&state, session_id, headers);
    match handlers::handle_notify_peek(ctx).await {
        Ok(queued) => (StatusCode::OK, axum::Json(queued)).into_response(),
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
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };
    (status, e.message).into_response()
}
