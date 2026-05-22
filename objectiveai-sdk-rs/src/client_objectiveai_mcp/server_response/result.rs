use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Tagged union of response shapes. `Ok` carries the MCP-shape JSON
/// the client's local objectiveai-mcp produced (e.g.
/// `ListToolsResult` for `McpToolsList`, `CallToolResult` for
/// `McpToolsCall`). `Error` carries a numeric `code` and a JSON
/// `message` whose shape mirrors [`crate::error::ResponseError`].
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "client_objectiveai_mcp.server_response.Result")]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Result {
    /// Success. `value` is the MCP-shape JSON object the server
    /// passes back to the proxy that triggered the request.
    Ok {
        value: serde_json::Value,
    },
    /// The request failed. With internally-tagged + struct-variant
    /// serde flattens the inner fields alongside the `type` tag —
    /// e.g. `{"id":"…","type":"error","code":404,"message":…}`.
    Error {
        code: u16,
        message: serde_json::Value,
    },
}
