use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Tagged union of standard MCP request shapes the API forwards down
/// the reverse-attach channel to the MCP-server layer of a calling
/// client's local `objectiveai-mcp`.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "client_objectiveai_mcp.server_request.Payload")]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Payload {
    McpToolsList(crate::mcp::tool::ListToolsRequest),
    McpToolsCall(crate::mcp::tool::CallToolRequestParams),
}
