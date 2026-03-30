use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "agent.claude_agent_sdk.Continuation")]
pub struct Continuation {
    pub upstream: super::Upstream,
    pub session_id: String,
    pub mcp_sessions: indexmap::IndexMap<String, String>,
}
