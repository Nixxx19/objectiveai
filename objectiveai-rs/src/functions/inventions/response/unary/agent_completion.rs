use crate::agent;
use crate::functions::inventions::response;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentCompletion {
    pub index: u64,
    #[serde(flatten)]
    pub inner: agent::completions::response::unary::AgentCompletion,
}

impl From<response::streaming::AgentCompletionChunk> for AgentCompletion {
    fn from(
        response::streaming::AgentCompletionChunk {
            index,
            inner,
        }: response::streaming::AgentCompletionChunk,
    ) -> Self {
        Self {
            index,
            inner: inner.into(),
        }
    }
}
