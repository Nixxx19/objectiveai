use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "functions.executions.response.streaming.Object")]
pub enum Object {
    #[serde(rename = "scalar.function.execution.chunk")]
    ScalarFunctionExecutionChunk,
    #[serde(rename = "vector.function.execution.chunk")]
    VectorFunctionExecutionChunk,
}
