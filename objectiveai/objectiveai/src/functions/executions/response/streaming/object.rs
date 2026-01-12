use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Object {
    #[serde(rename = "scalar.function.execution.chunk")]
    ScalarFunctionResponseChunk,
    #[serde(rename = "vector.function.execution.chunk")]
    VectorFunctionResponseChunk,
}
