//! Agent completion response type.

use crate::agent::completions::response;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// A complete agent completion response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, JsonSchema)]
#[schemars(rename = "agent.completions.response.unary.AgentCompletion")]
pub struct AgentCompletion {
    pub id: String,
    pub created: u64,
    pub messages: Vec<super::Message>,
    /// The object type (always "agent.completion").
    pub object: super::Object,
    pub usage: response::Usage,
    /// Upstream provider
    pub upstream: crate::agent::Upstream,
    /// Error details if this completion failed.
    pub error: Option<crate::error::ResponseError>,
    /// Continuation state for multi-turn conversations.
    pub continuation: Option<String>,
}

impl AgentCompletion {
    /// Normalize non-deterministic fields for test snapshot comparison.
    pub fn normalize_for_tests(&mut self) {
        self.id = String::new();
        self.created = 0;
        for msg in &mut self.messages {
            if let super::Message::Assistant(asst) = msg {
                asst.upstream_id = String::new();
                asst.created = 0;
            }
        }
    }
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
            continuation,
        }: response::streaming::AgentCompletionChunk,
    ) -> Self {
        Self {
            id,
            created,
            messages: messages.into_iter().map(Into::into).collect(),
            object: object.into(),
            usage: usage.unwrap_or_default(),
            upstream,
            error,
            continuation,
        }
    }
}
