use crate::chat::completions::response;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatCompletion {
    pub id: String,
    pub upstream_id: String,
    pub choices: Vec<super::Choice>,
    pub created: u64,
    pub model: String,
    pub upstream_model: String,
    pub object: super::Object,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
    pub usage: response::Usage,

    // openrouter fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}

impl From<response::streaming::ChatCompletionChunk> for ChatCompletion {
    fn from(
        response::streaming::ChatCompletionChunk {
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
            provider,
        }: response::streaming::ChatCompletionChunk,
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
            provider,
        }
    }
}
