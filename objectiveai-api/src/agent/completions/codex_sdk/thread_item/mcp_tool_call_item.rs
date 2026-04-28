use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct McpToolCallItem {
    pub id: String,
    pub server: String,
    pub tool: String,
    /// Arguments forwarded to the tool. The shape is defined by the MCP
    /// server's tool schema — opaque from our side.
    pub arguments: serde_json::Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<super::McpToolCallResult>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<super::McpToolCallError>,
    pub status: super::McpToolCallStatus,
}
