//! Requests addressed to the MCP-server protocol layer of the local
//! objectiveai-mcp — standard MCP `tools/list` and `tools/call`
//! shapes the API forwards on behalf of its upstream MCP clients.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// One of the standard MCP request shapes the API forwards down the
/// reverse-attach channel to the MCP-server layer of a calling
/// client's local `objectiveai-mcp`.
///
/// Wire shape (internally-tagged):
///
/// - `{"type": "mcp_tools_list", "cursor": "..."}`
/// - `{"type": "mcp_tools_call", "name": "...", "arguments": {...}, ...}`
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "client_objectiveai_mcp.server_request.Request")]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    McpToolsList(crate::mcp::tool::ListToolsRequest),
    McpToolsCall(crate::mcp::tool::CallToolRequestParams),
}
