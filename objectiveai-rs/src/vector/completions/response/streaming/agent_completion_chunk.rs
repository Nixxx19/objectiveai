//! Streaming agent completion chunk for vector completions.

use crate::{agent, error};
use serde::{Deserialize, Serialize};

/// A streaming agent completion chunk from a single LLM within a vector completion.
///
/// The `index` field is used to correlate chunks belonging to the same
/// underlying completion when accumulating via [`push`](Self::push).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentCompletionChunk {
    /// Index used to correlate chunks from the same completion.
    pub index: u64,
    /// The underlying agent completion chunk.
    #[serde(flatten)]
    pub inner: agent::completions::response::streaming::AgentCompletionChunk,
    /// Error details if this completion failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<error::ResponseError>,
}

impl AgentCompletionChunk {
    pub fn push(&mut self, other: &AgentCompletionChunk) {
        self.inner.push(&other.inner);
        match (&mut self.error, &other.error) {
            (None, Some(other_error)) => {
                self.error = Some(other_error.clone());
            }
            _ => {}
        }
    }
}
