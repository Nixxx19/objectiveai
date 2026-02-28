use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SDKStatusMessageType {
    System,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SDKStatusMessageSubtype {
    Status,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SDKStatusValue {
    Compacting,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SDKStatusMessage {
    pub r#type: SDKStatusMessageType,
    pub subtype: SDKStatusMessageSubtype,
    pub status: Option<SDKStatusValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission_mode: Option<super::PermissionMode>,
    pub uuid: String,
    pub session_id: String,
}
