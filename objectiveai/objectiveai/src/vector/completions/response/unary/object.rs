use crate::vector::completions::response;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum Object {
    #[serde(rename = "vector.completion")]
    #[default]
    VectorCompletion,
}

impl From<response::streaming::Object> for Object {
    fn from(object: response::streaming::Object) -> Self {
        match object {
            response::streaming::Object::VectorCompletionChunk => {
                Object::VectorCompletion
            }
        }
    }
}
