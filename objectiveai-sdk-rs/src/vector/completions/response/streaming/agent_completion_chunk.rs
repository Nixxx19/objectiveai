//! Streaming agent completion chunk for vector completions.

use crate::agent;
use crate::agent::completions::response::streaming::AgentCompletionIds;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// A streaming agent completion chunk from a single agent within a vector completion.
///
/// The `index` field is used to correlate chunks belonging to the same
/// underlying completion when accumulating via [`push`](Self::push).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, JsonSchema, arbitrary::Arbitrary)]
#[schemars(rename = "vector.completions.response.streaming.AgentCompletionChunk")]
pub struct AgentCompletionChunk {
    /// Index used to correlate chunks from the same completion.
    #[arbitrary(with = crate::arbitrary_util::arbitrary_u64)]
    pub index: u64,
    /// The underlying agent completion chunk.
    #[serde(flatten)]
    pub inner: agent::completions::response::streaming::AgentCompletionChunk,
}

impl AgentCompletionIds for AgentCompletionChunk {
    fn agent_completion_ids(&self) -> impl Iterator<Item = &str> {
        self.inner.agent_completion_ids()
    }
}

impl AgentCompletionChunk {
    pub fn push(&mut self, other: &AgentCompletionChunk) {
        self.inner.push(&other.inner);
    }

    /// Produces log files for this agent completion within a vector completion.
    ///
    /// Returns `(reference, files)` where `reference` is a [`LogReference`]
    /// carrying `"index"`. Files are written under `agent/completions/`
    /// (shared with standalone agent completions).
    #[cfg(feature = "filesystem")]
    pub fn produce_files(
        &self,
    ) -> (crate::filesystem::logs::LogReference, Vec<crate::filesystem::logs::LogFile>) {
        use crate::filesystem::logs::LogReference;
        let (mut reference, files) = match self.inner.produce_files() {
            Some((reference, files)) => (reference, files),
            None => {
                let mut r = LogReference::new(String::new());
                r.index = Some(self.index);
                return (r, Vec::new());
            }
        };
        reference.index = Some(self.index);
        (reference, files)
    }
}
