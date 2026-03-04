//! Agent completion response type.

use crate::agent::completions::response;
use serde::{Deserialize, Serialize};

/// A complete agent completion response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AgentCompletion {
    pub id: String,
    pub created: u64,
    pub messages: Vec<super::Message>,
    /// The object type (always "agent.completion").
    pub object: super::Object,
    /// Token usage (only present in the final chunk).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<response::Usage>,
    /// Upstream provider
    pub upstream: crate::agent::Upstream,
    /// Error details if this completion failed.
    pub error: Option<crate::error::ResponseError>,
}

impl From<response::streaming::AgentCompletionChunk> for AgentCompletion {
    fn from(
        response::streaming::AgentCompletionChunk {
            id,
            created,
            messages,
            object,
            usage,
            upstream,
            error,
        }: response::streaming::AgentCompletionChunk,
    ) -> Self {
        Self {
            id,
            created,
            messages: messages.into_iter().map(Into::into).collect(),
            object: object.into(),
            usage,
            upstream,
            error,
        }
    }
}
