use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SDKCompactBoundaryMessageType {
    System,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SDKCompactBoundaryMessageSubtype {
    CompactBoundary,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CompactTrigger {
    Manual,
    Auto,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CompactMetadata {
    pub trigger: CompactTrigger,
    pub pre_tokens: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SDKCompactBoundaryMessage {
    pub r#type: SDKCompactBoundaryMessageType,
    pub subtype: SDKCompactBoundaryMessageSubtype,
    pub compact_metadata: CompactMetadata,
    pub uuid: String,
    pub session_id: String,
}
