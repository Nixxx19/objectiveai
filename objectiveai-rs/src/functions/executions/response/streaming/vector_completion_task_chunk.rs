use crate::{error, vector};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, JsonSchema)]
#[schemars(rename = "functions.executions.response.streaming.VectorCompletionTaskChunk")]
pub struct VectorCompletionTaskChunk {
    pub index: u64,
    pub task_index: u64,
    pub task_path: Vec<u64>,
    #[serde(flatten)]
    pub inner: vector::completions::response::streaming::VectorCompletionChunk,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<error::ResponseError>,
}

impl VectorCompletionTaskChunk {
    pub fn push(&mut self, other: &VectorCompletionTaskChunk) {
        self.inner.push(&other.inner);
        if let Some(error) = &other.error {
            self.error = Some(error.clone());
        }
    }
}
