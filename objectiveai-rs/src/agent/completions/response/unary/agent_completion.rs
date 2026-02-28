//! Agent completion response type.

use crate::agent::completions::response;
use serde::{Deserialize, Serialize};

/// A complete agent completion response.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentCompletion {
    /// ObjectiveAI's unique identifier for this completion.
    pub id: String,
    /// The upstream provider's identifier.
    pub upstream_id: String,
    /// The generated choices.
    pub choices: Vec<super::Choice>,
    /// Unix timestamp when the completion was created.
    pub created: u64,
    /// The Ensemble LLM ID used.
    pub model: String,
    /// The upstream model identifier.
    pub upstream_model: String,
    /// The object type (always "agent.completion").
    pub object: super::Object,
    /// The service tier used, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,
    /// A fingerprint of the model configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
    /// Token usage statistics.
    pub usage: response::Usage,
    /// Upstream provider
    pub upstream: crate::upstream::Upstream,

    /// The provider that served the request (OpenRouter-specific).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}

impl From<response::streaming::AgentCompletionChunk> for AgentCompletion {
    fn from(
        response::streaming::AgentCompletionChunk {
            id,
            upstream_id,
            choices,
            created,
            model,
            upstream_model,
            object,
            service_tier,
            system_fingerprint,
            usage,
            upstream,
            provider,
        }: response::streaming::AgentCompletionChunk,
    ) -> Self {
        Self {
            id,
            upstream_id,
            choices: choices.into_iter().map(super::Choice::from).collect(),
            created,
            model,
            upstream_model,
            object: object.into(),
            service_tier,
            system_fingerprint,
            usage: usage.unwrap_or_default(),
            upstream,
            provider,
        }
    }
}
