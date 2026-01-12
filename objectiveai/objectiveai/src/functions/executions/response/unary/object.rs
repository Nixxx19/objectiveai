use crate::functions::executions::response;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Object {
    #[serde(rename = "scalar.function.execution")]
    ScalarFunctionResponse,
    #[serde(rename = "vector.function.execution")]
    VectorFunctionResponse,
}

impl From<response::streaming::Object> for Object {
    fn from(value: response::streaming::Object) -> Self {
        match value {
            response::streaming::Object::ScalarFunctionResponseChunk => {
                Object::ScalarFunctionResponse
            }
            response::streaming::Object::VectorFunctionResponseChunk => {
                Object::VectorFunctionResponse
            }
        }
    }
}
