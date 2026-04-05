use crate::{agent, error, laboratories::executions::response};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// A single builder agent completion within a laboratory execution (non-streaming).
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
#[schemars(rename = "laboratories.executions.response.unary.Builder")]
pub struct Builder {
    /// Builder index (0-based).
    pub index: u64,
    /// Container index (0-based).
    pub container_index: u64,
    #[serde(flatten)]
    pub inner: agent::completions::response::unary::AgentCompletion,
    pub error: Option<error::ResponseError>,
}

impl From<response::streaming::BuilderChunk> for Builder {
    fn from(
        response::streaming::BuilderChunk {
            index,
            container_index,
            inner,
            error,
        }: response::streaming::BuilderChunk,
    ) -> Self {
        Self {
            index,
            container_index,
            inner: inner.into(),
            error,
        }
    }
}
