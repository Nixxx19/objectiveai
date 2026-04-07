use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SDKHookResponseMessageType {
    System,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SDKHookResponseMessageSubtype {
    HookResponse,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HookOutcome {
    Success,
    Error,
    Cancelled,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SDKHookResponseMessage {
    pub r#type: SDKHookResponseMessageType,
    pub subtype: SDKHookResponseMessageSubtype,
    pub hook_id: String,
    pub hook_name: String,
    pub hook_event: String,
    pub output: String,
    pub stdout: String,
    pub stderr: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    pub outcome: HookOutcome,
    pub uuid: String,
    pub session_id: String,
}
