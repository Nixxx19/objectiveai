use crate::functions;
use crate::agent::completions::response::streaming::AgentCompletionIds;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, arbitrary::Arbitrary)]
#[schemars(rename = "functions.inventions.recursive.response.streaming.FunctionInventionChunk")]
pub struct FunctionInventionChunk {
    #[arbitrary(with = crate::arbitrary_util::arbitrary_u64)]
    pub index: u64,
    #[serde(flatten)]
    pub inner:
        functions::inventions::response::streaming::FunctionInventionChunk,
}

impl AgentCompletionIds for FunctionInventionChunk {
    fn agent_completion_ids(&self) -> impl Iterator<Item = &str> {
        self.inner.agent_completion_ids()
    }
}

impl FunctionInventionChunk {
    pub fn push(&mut self, other: &FunctionInventionChunk) {
        self.inner.push(&other.inner);
    }

    /// Produces log files for this invention within a recursive invention.
    ///
    /// Returns `(reference, files)` where `reference` carries `index`.
    /// Files are written under `functions/inventions/`.
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
