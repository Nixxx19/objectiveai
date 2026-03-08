use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct RemoteFunctionPath {
    pub remote: super::Remote,
    pub owner: String,
    pub repository: String,
    pub commit: String,
}
