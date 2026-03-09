use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Object {
    #[serde(rename = "alpha.scalar.function.invention.recursive.chunk")]
    AlphaScalarFunctionInventionRecursiveChunk,
    #[serde(rename = "alpha.vector.function.invention.recursive.chunk")]
    AlphaVectorFunctionInventionRecursiveChunk,
}

impl From<Object> for crate::functions::inventions::response::streaming::Object {
    fn from(object: Object) -> Self {
        match object {
            Object::AlphaScalarFunctionInventionRecursiveChunk => {
                Self::AlphaScalarFunctionInventionChunk
            }
            Object::AlphaVectorFunctionInventionRecursiveChunk => {
                Self::AlphaVectorFunctionInventionChunk
            }
        }
    }
}
