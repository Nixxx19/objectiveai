use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SDKAuthStatusMessageType {
    AuthStatus,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SDKAuthStatusMessage {
    pub r#type: SDKAuthStatusMessageType,
    #[serde(rename = "isAuthenticating")]
    pub is_authenticating: bool,
    pub output: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    pub uuid: String,
    pub session_id: String,
}
