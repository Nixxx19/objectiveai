use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, JsonSchema)]
#[schemars(rename = "functions.RemoteFunctionPath")]
pub struct RemoteFunctionPath {
    pub remote: super::Remote,
    pub owner: String,
    pub repository: String,
    pub commit: String,
}
