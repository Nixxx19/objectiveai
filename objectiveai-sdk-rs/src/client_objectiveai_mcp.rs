//! Client-side ObjectiveAI MCP request envelope.
//!
//! A discriminated union the API uses (over the reverse-attach
//! transport from #193) to push a single call into a calling
//! client's local `objectiveai-mcp` process. Each variant wraps an
//! existing request type — the notify type lives in
//! `agent::completions::request`, the two MCP types live in
//! `mcp::tool`.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// One of the three call shapes the API can push down the
/// reverse-attach channel into a calling client's `objectiveai-mcp`.
///
/// Wire shape (internally-tagged):
///
/// - `{"type": "agent_completion_notify", "response_id": "...", "content": ...}`
/// - `{"type": "mcp_tools_list",          "cursor": "..."}`
/// - `{"type": "mcp_tools_call",          "name": "...", "arguments": {...}, ...}`
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "client_objectiveai_mcp.Request")]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    AgentCompletionNotify(crate::agent::completions::request::AgentCompletionNotifyParams),
    McpToolsList(crate::mcp::tool::ListToolsRequest),
    McpToolsCall(crate::mcp::tool::CallToolRequestParams),
}
