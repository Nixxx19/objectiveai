use crate::agent;
use crate::agent::completions::response::streaming::AgentCompletionIds;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Streaming chunk for a single builder agent completion within a laboratory execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, JsonSchema, arbitrary::Arbitrary)]
#[schemars(rename = "laboratories.executions.response.streaming.BuilderChunk")]
pub struct BuilderChunk {
    /// Builder index (0-based).
    #[arbitrary(with = crate::arbitrary_util::arbitrary_u64)]
    pub index: u64,
    /// Agent index (0-based).
    #[arbitrary(with = crate::arbitrary_util::arbitrary_u64)]
    pub agent_index: u64,
    #[serde(flatten)]
    pub inner: agent::completions::response::streaming::AgentCompletionChunk,
}

impl AgentCompletionIds for BuilderChunk {
    fn agent_completion_ids(&self) -> impl Iterator<Item = &str> {
        self.inner.agent_completion_ids()
    }
}

impl BuilderChunk {
    pub fn push(&mut self, other: &BuilderChunk) {
        self.inner.push(&other.inner);
    }

    /// Produces log files for this builder completion.
    ///
    /// Returns `(reference, files)` where `reference` includes `"index"` and `"agent_index"`.
    /// Files are written under `agent/completions/`.
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
                r.agent_index = Some(self.agent_index);
                return (r, Vec::new());
            }
        };
        reference.index = Some(self.index);
        reference.agent_index = Some(self.agent_index);
        (reference, files)
    }
}
