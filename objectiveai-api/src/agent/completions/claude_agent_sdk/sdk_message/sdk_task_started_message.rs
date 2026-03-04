use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SDKTaskStartedMessageType {
    System,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SDKTaskStartedMessageSubtype {
    TaskStarted,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SDKTaskStartedMessage {
    pub r#type: SDKTaskStartedMessageType,
    pub subtype: SDKTaskStartedMessageSubtype,
    pub task_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_use_id: Option<String>,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_type: Option<String>,
    pub uuid: String,
    pub session_id: String,
}
