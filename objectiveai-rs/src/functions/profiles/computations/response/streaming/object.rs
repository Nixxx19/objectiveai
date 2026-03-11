use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "functions.profiles.computations.response.streaming.Object")]
pub enum Object {
    #[serde(rename = "function.profile.computation.chunk")]
    FunctionProfileComputationChunk,
}
