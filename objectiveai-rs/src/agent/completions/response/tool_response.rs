use crate::agent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolResponse {
    pub role: ToolRole,
    pub index: u64,
    #[serde(flatten)]
    pub inner: agent::completions::message::ToolMessage,
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq,
)]
pub enum ToolRole {
    #[serde(rename = "tool")]
    #[default]
    Tool,
}
