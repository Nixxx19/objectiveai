//! Requests addressed to the client-app layer of the local
//! objectiveai-mcp — non-MCP-protocol pushes (e.g. surface a user
//! message into a running agent completion).

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// One of the request shapes the API can push down the
/// reverse-attach channel to the client-app layer of a calling
/// client's local `objectiveai-mcp`.
///
/// Wire shape (internally-tagged):
///
/// - `{"type": "agent_completion_notify", "response_id": "...", "content": ...}`
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "client_objectiveai_mcp.client_request.Request")]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Request {
    AgentCompletionNotify(crate::agent::completions::request::AgentCompletionNotifyParams),
}
