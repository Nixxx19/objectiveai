use crate::functions::inventions::response;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Object {
    #[serde(rename = "alpha.scalar.function.invention")]
    AlphaScalarFunctionInvention,
    #[serde(rename = "alpha.vector.function.invention")]
    AlphaVectorFunctionInvention,
}

impl From<response::streaming::Object> for Object {
    fn from(value: response::streaming::Object) -> Self {
        match value {
            response::streaming::Object::AlphaScalarFunctionInventionChunk => {
                Object::AlphaScalarFunctionInvention
            }
            response::streaming::Object::AlphaVectorFunctionInventionChunk => {
                Object::AlphaVectorFunctionInvention
            }
        }
    }
}
