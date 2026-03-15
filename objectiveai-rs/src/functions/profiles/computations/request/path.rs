use crate::functions::Remote;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "functions.profiles.computations.request.FunctionRemoteRequestPath")]
pub struct FunctionRemoteRequestPath {
    pub fremote: Remote,
    pub fowner: String,
    pub frepository: String,
    pub fcommit: Option<String>,
}
