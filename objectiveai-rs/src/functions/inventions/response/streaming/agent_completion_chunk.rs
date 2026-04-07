use crate::agent;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, JsonSchema, arbitrary::Arbitrary)]
#[schemars(rename = "functions.inventions.response.streaming.AgentCompletionChunk")]
pub struct AgentCompletionChunk {
    #[arbitrary(with = crate::arbitrary_util::arbitrary_u64)]
    pub index: u64,
    #[serde(flatten)]
    pub inner: agent::completions::response::streaming::AgentCompletionChunk,
}

impl AgentCompletionChunk {
    pub fn push(&mut self, other: &AgentCompletionChunk) {
        self.inner.push(&other.inner);
    }
}
