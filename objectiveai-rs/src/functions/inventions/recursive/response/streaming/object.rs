use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Object {
    #[serde(rename = "alpha.scalar.function.invention.recursive.chunk")]
    AlphaScalarFunctionInventionRecursiveChunk,
    #[serde(rename = "alpha.vector.function.invention.recursive.chunk")]
    AlphaVectorFunctionInventionRecursiveChunk,
}
