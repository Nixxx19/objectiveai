use crate::{error, vector};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, JsonSchema, arbitrary::Arbitrary)]
#[schemars(rename = "functions.executions.response.streaming.VectorCompletionTaskChunk")]
pub struct VectorCompletionTaskChunk {
    #[arbitrary(with = crate::arbitrary_util::arbitrary_u64)]
    pub index: u64,
    #[arbitrary(with = crate::arbitrary_util::arbitrary_u64)]
    pub task_index: u64,
    #[arbitrary(with = crate::arbitrary_util::arbitrary_vec_u64)]
    pub task_path: Vec<u64>,
    #[serde(flatten)]
    pub inner: vector::completions::response::streaming::VectorCompletionChunk,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
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
