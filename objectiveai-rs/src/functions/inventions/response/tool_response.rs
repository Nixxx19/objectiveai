use crate::chat;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResponse {
    pub role: ToolRole,
    pub index: u64,
    #[serde(flatten)]
    pub inner: chat::completions::request::ToolMessage,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ToolRole {
    #[serde(rename = "tool")]
    Tool,
}
