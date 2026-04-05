use crate::{agent, error, functions, laboratories::executions::response};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// A single evaluation agent completion within a laboratory execution (non-streaming).
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
#[schemars(rename = "laboratories.executions.response.unary.Evaluation")]
pub struct Evaluation {
    /// Evaluation index (0-based).
    pub index: u64,
    /// Container index (0-based).
    pub container_index: u64,
    #[serde(flatten)]
    pub inner: agent::completions::response::unary::AgentCompletion,
    pub output: Option<functions::expression::InputValue>,
    pub error: Option<error::ResponseError>,
}

impl From<response::streaming::EvaluationChunk> for Evaluation {
    fn from(
        response::streaming::EvaluationChunk {
            index,
            container_index,
            inner,
            output,
            error,
        }: response::streaming::EvaluationChunk,
    ) -> Self {
        Self {
            index,
            container_index,
            inner: inner.into(),
            output,
            error,
        }
    }
}
