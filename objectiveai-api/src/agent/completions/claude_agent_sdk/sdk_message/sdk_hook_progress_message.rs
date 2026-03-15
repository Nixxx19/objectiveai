use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SDKHookProgressMessageType {
    System,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SDKHookProgressMessageSubtype {
    HookProgress,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SDKHookProgressMessage {
    pub r#type: SDKHookProgressMessageType,
    pub subtype: SDKHookProgressMessageSubtype,
    pub hook_id: String,
    pub hook_name: String,
    pub hook_event: String,
    pub stdout: String,
    pub stderr: String,
    pub output: String,
    pub uuid: String,
    pub session_id: String,
}
