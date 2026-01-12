use serde::{Deserialize, Serialize};

pub struct FunctionRemoteRequestPath {
    pub fowner: String,
    pub frepository: String,
    pub fcommit: Option<String>,
}
