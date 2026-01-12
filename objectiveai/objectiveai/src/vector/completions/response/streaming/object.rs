use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum Object {
    #[serde(rename = "vector.completion.chunk")]
    #[default]
    VectorCompletionChunk,
}
