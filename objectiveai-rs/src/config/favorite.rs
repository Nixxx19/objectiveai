use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Favorite {
    #[serde(flatten)]
    pub path: crate::RemotePathCommitOptional,
    pub note: String,
}
