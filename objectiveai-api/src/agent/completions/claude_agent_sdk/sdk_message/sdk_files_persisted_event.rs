use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SDKFilesPersistedEventType {
    System,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SDKFilesPersistedEventSubtype {
    FilesPersisted,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PersistedFile {
    pub filename: String,
    pub file_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FailedFile {
    pub filename: String,
    pub error: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SDKFilesPersistedEvent {
    pub r#type: SDKFilesPersistedEventType,
    pub subtype: SDKFilesPersistedEventSubtype,
    pub files: Vec<PersistedFile>,
    pub failed: Vec<FailedFile>,
    pub processed_at: String,
    pub uuid: String,
    pub session_id: String,
}
