use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaMessageRole {
    Assistant,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaMessageType {
    Message,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaMessage {
    pub id: String,
    pub container: Option<super::BetaContainer>,
    pub content: Vec<super::super::beta_content_block::BetaContentBlock>,
    pub context_management: Option<super::BetaContextManagementResponse>,
    pub model: String,
    pub role: BetaMessageRole,
    pub stop_reason: Option<super::BetaStopReason>,
    pub stop_sequence: Option<String>,
    pub r#type: BetaMessageType,
    pub usage: super::super::beta_usage::BetaUsage,
}
