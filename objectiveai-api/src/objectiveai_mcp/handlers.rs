//! Typed delegate functions, one per MCP route. Every body is
//! `todo!()` for now — this file defines the stable shape future
//! commits will fill in.

use crate::objectiveai_mcp::context::McpRequestContext;
use objectiveai_sdk::mcp::initialize_result::InitializeResult;
use objectiveai_sdk::mcp::resource::{
    ListResourcesRequest, ListResourcesResult, ReadResourceRequestParams,
    ReadResourceResult,
};
use objectiveai_sdk::mcp::tool::{
    CallToolRequestParams, CallToolResult, ContentBlock, ListToolsRequest,
    ListToolsResult,
};
use serde::{Deserialize, Serialize};

/// Common error shape every delegate returns. The route layer
/// renders this into either a JSON-RPC error envelope (under
/// `POST /`) or an HTTP status response (for `/notify` + `DELETE`).
///
/// Codes follow JSON-RPC conventions even on the non-JSON-RPC routes
/// so the route layer's shape is uniform — see `routes::mcp_error_to_http`.
#[derive(Debug)]
pub struct McpError {
    pub code: i64,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// Minimal `initialize` params struct — only `protocolVersion` is
/// load-bearing for the proxy ([`objectiveai-mcp-proxy/src/mcp.rs:246-273`])
/// and the same is true here. `clientInfo` / `capabilities` arrive
/// on the wire and serde drops them.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeRequestParams {
    pub protocol_version: String,
}

// ────────────────────────────────────────────────────────────────
// JSON-RPC method delegates (POST /objectiveai-mcp/{session_id})
// ────────────────────────────────────────────────────────────────

pub async fn handle_initialize(
    ctx: McpRequestContext,
    params: InitializeRequestParams,
) -> Result<InitializeResult, McpError> {
    let _ = (ctx, params);
    todo!("API MCP: initialize — synthesize ServerInfo + capabilities + advertise list_changed");
}

pub async fn handle_ping(ctx: McpRequestContext) -> Result<(), McpError> {
    let _ = ctx;
    todo!("API MCP: ping — return empty result");
}

pub async fn handle_tools_list(
    ctx: McpRequestContext,
    params: ListToolsRequest,
) -> Result<ListToolsResult, McpError> {
    let _ = (ctx, params);
    todo!("API MCP: tools/list — forward to CLI via send_server_request, return aggregated list");
}

pub async fn handle_tools_call(
    ctx: McpRequestContext,
    params: CallToolRequestParams,
) -> Result<CallToolResult, McpError> {
    let _ = (ctx, params);
    todo!("API MCP: tools/call — forward to CLI via send_server_request, prepend pending notifications");
}

pub async fn handle_resources_list(
    ctx: McpRequestContext,
    params: ListResourcesRequest,
) -> Result<ListResourcesResult, McpError> {
    let _ = (ctx, params);
    todo!("API MCP: resources/list — forward to CLI via send_server_request");
}

pub async fn handle_resources_read(
    ctx: McpRequestContext,
    params: ReadResourceRequestParams,
) -> Result<ReadResourceResult, McpError> {
    let _ = (ctx, params);
    todo!("API MCP: resources/read — forward to CLI via send_server_request");
}

// ────────────────────────────────────────────────────────────────
// Session lifecycle (DELETE /objectiveai-mcp/{session_id})
// ────────────────────────────────────────────────────────────────

pub async fn handle_session_terminate(
    ctx: McpRequestContext,
) -> Result<(), McpError> {
    let _ = ctx;
    todo!("API MCP: DELETE — terminate the WS session, drop reverse-attach registration");
}

// ────────────────────────────────────────────────────────────────
// /notify extensions (ObjectiveAI-specific, mirrors proxy's POST
// /notify / GET /notify / GET /notify/queued)
// ────────────────────────────────────────────────────────────────

pub async fn handle_notify_enqueue(
    ctx: McpRequestContext,
    blocks: Vec<ContentBlock>,
) -> Result<(), McpError> {
    let _ = (ctx, blocks);
    todo!("API MCP: POST /notify — append to pending-notifications queue for this session");
}

pub async fn handle_notify_drain(
    ctx: McpRequestContext,
) -> Result<Vec<ContentBlock>, McpError> {
    let _ = ctx;
    todo!("API MCP: GET /notify — atomic drain of pending-notifications queue");
}

pub async fn handle_notify_peek(
    ctx: McpRequestContext,
) -> Result<bool, McpError> {
    let _ = ctx;
    todo!("API MCP: GET /notify/queued — true iff queue non-empty, no drain");
}
