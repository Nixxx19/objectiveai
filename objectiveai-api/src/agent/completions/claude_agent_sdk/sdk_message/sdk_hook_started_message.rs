use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SDKHookStartedMessageType {
    System,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SDKHookStartedMessageSubtype {
    HookStarted,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SDKHookStartedMessage {
    pub r#type: SDKHookStartedMessageType,
    pub subtype: SDKHookStartedMessageSubtype,
    pub hook_id: String,
    pub hook_name: String,
    pub hook_event: String,
    pub uuid: String,
    pub session_id: String,
}
