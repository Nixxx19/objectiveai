use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "FunctionsExecutionsResponseStreamingObject")]
pub enum Object {
    #[serde(rename = "scalar.function.execution.chunk")]
    ScalarFunctionExecutionChunk,
    #[serde(rename = "vector.function.execution.chunk")]
    VectorFunctionExecutionChunk,
}
