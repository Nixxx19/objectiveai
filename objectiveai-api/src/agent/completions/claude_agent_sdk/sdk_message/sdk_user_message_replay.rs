use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SDKUserMessageReplayType {
    User,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SDKUserMessageReplay {
    #[serde(rename = "type")]
    pub r#type: SDKUserMessageReplayType,
    pub message: super::MessageParam,
    pub parent_tool_use_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "isSynthetic")]
    pub is_synthetic: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_use_result: Option<serde_json::Value>,
    pub uuid: String,
    pub session_id: String,
    #[serde(rename = "isReplay")]
    pub is_replay: bool,
}
