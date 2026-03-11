use crate::agent;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "AgentCompletionsResponseToolResponse")]
pub struct ToolResponse {
    pub role: ToolRole,
    pub index: u64,
    #[serde(flatten)]
    pub inner: agent::completions::message::ToolMessage,
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq, JsonSchema,
)]
#[schemars(rename = "AgentCompletionsResponseToolRole")]
pub enum ToolRole {
    #[serde(rename = "tool")]
    #[default]
    Tool,
}
