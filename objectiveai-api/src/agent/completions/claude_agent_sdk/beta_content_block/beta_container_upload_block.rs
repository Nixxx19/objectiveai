use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaContainerUploadBlockType {
    ContainerUpload,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaContainerUploadBlock {
    pub file_id: String,
    pub r#type: BetaContainerUploadBlockType,
}
