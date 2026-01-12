use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionRemoteProfileInlineRequestPath {
    pub fowner: String,
    pub frepository: String,
    pub fcommit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInlineProfileRemoteRequestPath {
    pub powner: String,
    pub prepository: String,
    pub pcommit: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionRemoteProfileRemoteRequestPath {
    pub fowner: String,
    pub frepository: String,
    pub fcommit: Option<String>,
    pub powner: String,
    pub prepository: String,
    pub pcommit: Option<String>,
}
