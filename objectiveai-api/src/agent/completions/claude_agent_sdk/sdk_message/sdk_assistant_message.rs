use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SDKAssistantMessage {
    pub r#type: SDKAssistantMessageType,
    pub message: super::super::beta_message::BetaMessage,
    pub parent_tool_use_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<SDKAssistantMessageError>,
    pub uuid: String,
    pub session_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SDKAssistantMessageType {
    Assistant,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SDKAssistantMessageError {
    AuthenticationFailed,
    BillingError,
    RateLimit,
    InvalidRequest,
    ServerError,
    Unknown,
    MaxOutputTokens,
}
