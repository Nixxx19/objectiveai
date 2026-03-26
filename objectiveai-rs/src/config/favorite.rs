use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Favorite {
    #[serde(flatten)]
    pub path: crate::RemotePathCommitOptional,
    pub note: String,
}

impl Favorite {
    pub fn path(&self) -> &crate::RemotePathCommitOptional {
        &self.path
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairFavorite {
    pub function: crate::RemotePathCommitOptional,
    pub profile: crate::RemotePathCommitOptional,
    pub note: String,
}
