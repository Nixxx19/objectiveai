use crate::agent;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
#[schemars(rename = "FunctionsInventionsResponseStreamingAgentCompletionChunk")]
pub struct AgentCompletionChunk {
    pub index: u64,
    #[serde(flatten)]
    pub inner: agent::completions::response::streaming::AgentCompletionChunk,
}

impl AgentCompletionChunk {
    pub fn push(&mut self, other: &AgentCompletionChunk) {
        self.inner.push(&other.inner);
    }
}
