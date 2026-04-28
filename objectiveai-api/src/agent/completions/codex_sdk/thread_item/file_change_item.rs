use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileChangeItem {
    pub id: String,
    pub changes: Vec<super::FileUpdateChange>,
    pub status: super::PatchApplyStatus,
}
