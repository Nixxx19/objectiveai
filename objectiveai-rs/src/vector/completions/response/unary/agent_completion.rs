//! Agent completion wrapper for vector completions.

use crate::{agent, error, vector::completions::response};
use serde::{Deserialize, Serialize};

/// A agent completion from a single LLM within a vector completion.
///
/// Wraps the standard agent completion response with an index to identify
/// which LLM in the ensemble produced it, and an optional error if the
/// completion failed.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentCompletion {
    /// Index of this completion within the vector completion.
    pub index: u64,
    /// The underlying agent completion response.
    #[serde(flatten)]
    pub inner: agent::completions::response::unary::AgentCompletion,
    /// Error details if this completion failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<error::ResponseError>,
}

impl From<response::streaming::AgentCompletionChunk> for AgentCompletion {
    fn from(
        response::streaming::AgentCompletionChunk {
            index,
            inner,
            error,
        }: response::streaming::AgentCompletionChunk,
    ) -> Self {
        Self {
            index,
            inner: agent::completions::response::unary::AgentCompletion::from(
                inner,
            ),
            error,
        }
    }
}
