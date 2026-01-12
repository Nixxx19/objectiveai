use crate::functions::compute_profile::response;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Object {
    #[serde(rename = "function.compute.profile")]
    FunctionOptimizeResponse,
}

impl From<response::streaming::Object> for Object {
    fn from(value: response::streaming::Object) -> Self {
        match value {
            response::streaming::Object::FunctionOptimizeResponseChunk => {
                Self::FunctionOptimizeResponse
            }
        }
    }
}
