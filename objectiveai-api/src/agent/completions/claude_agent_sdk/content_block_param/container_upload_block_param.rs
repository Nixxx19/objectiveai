use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContainerUploadBlockParamType {
    ContainerUpload,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ContainerUploadBlockParam {
    pub file_id: String,
    pub r#type: ContainerUploadBlockParamType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<super::CacheControlEphemeral>,
}
