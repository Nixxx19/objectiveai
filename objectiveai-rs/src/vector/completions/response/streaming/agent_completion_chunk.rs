//! Streaming agent completion chunk for vector completions.

use crate::agent;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// A streaming agent completion chunk from a single agent within a vector completion.
///
/// The `index` field is used to correlate chunks belonging to the same
/// underlying completion when accumulating via [`push`](Self::push).
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
#[schemars(rename = "VectorCompletionsResponseStreamingAgentCompletionChunk")]
pub struct AgentCompletionChunk {
    /// Index used to correlate chunks from the same completion.
    pub index: u64,
    /// The underlying agent completion chunk.
    #[serde(flatten)]
    pub inner: agent::completions::response::streaming::AgentCompletionChunk,
}

impl AgentCompletionChunk {
    pub fn push(&mut self, other: &AgentCompletionChunk) {
        self.inner.push(&other.inner);
    }
}
