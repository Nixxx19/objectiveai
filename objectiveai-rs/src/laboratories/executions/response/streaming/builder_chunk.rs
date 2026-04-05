use crate::{agent, error};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Streaming chunk for a single builder agent completion within a laboratory execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, JsonSchema, arbitrary::Arbitrary)]
#[schemars(rename = "laboratories.executions.response.streaming.BuilderChunk")]
pub struct BuilderChunk {
    /// Builder index (0-based).
    #[arbitrary(with = crate::arbitrary_util::arbitrary_u64)]
    pub index: u64,
    /// Container index (0-based).
    #[arbitrary(with = crate::arbitrary_util::arbitrary_u64)]
    pub agent_index: u64,
    #[serde(flatten)]
    pub inner: agent::completions::response::streaming::AgentCompletionChunk,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub error: Option<error::ResponseError>,
}

impl BuilderChunk {
    pub fn push(&mut self, other: &BuilderChunk) {
        self.inner.push(&other.inner);
        if let Some(error) = &other.error {
            self.error = Some(error.clone());
        }
    }
}
