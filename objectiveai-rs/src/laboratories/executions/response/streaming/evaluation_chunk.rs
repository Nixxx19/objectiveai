use crate::{agent, error, functions};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Streaming chunk for a single evaluation agent completion within a laboratory execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, JsonSchema, arbitrary::Arbitrary)]
#[schemars(rename = "laboratories.executions.response.streaming.EvaluationChunk")]
pub struct EvaluationChunk {
    /// Evaluation index (0-based).
    #[arbitrary(with = crate::arbitrary_util::arbitrary_u64)]
    pub index: u64,
    /// Container index (0-based).
    #[arbitrary(with = crate::arbitrary_util::arbitrary_u64)]
    pub container_index: u64,
    #[serde(flatten)]
    pub inner: agent::completions::response::streaming::AgentCompletionChunk,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub output: Option<functions::expression::InputValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub error: Option<error::ResponseError>,
}

impl EvaluationChunk {
    pub fn push(&mut self, other: &EvaluationChunk) {
        self.inner.push(&other.inner);
        if let Some(output) = &other.output {
            self.output = Some(output.clone());
        }
        if let Some(error) = &other.error {
            self.error = Some(error.clone());
        }
    }
}
