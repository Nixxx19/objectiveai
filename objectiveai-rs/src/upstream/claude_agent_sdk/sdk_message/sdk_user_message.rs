use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SDKUserMessageType {
    User,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MessageParamRole {
    User,
    Assistant,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum MessageParamContent {
    String(String),
    Blocks(Vec<super::super::content_block_param::ContentBlockParam>),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MessageParam {
    pub content: MessageParamContent,
    pub role: MessageParamRole,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SDKUserMessage {
    pub r#type: SDKUserMessageType,
    pub message: MessageParam,
    pub parent_tool_use_id: Option<String>,
    #[serde(rename = "isSynthetic", skip_serializing_if = "Option::is_none")]
    pub is_synthetic: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_use_result: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    pub session_id: String,
}
