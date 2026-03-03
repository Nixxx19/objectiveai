//! Chat completion request parameters for OpenRouter.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Chat completion request parameters formatted for the OpenRouter API.
///
/// Combines parameters from both the Agent configuration and the
/// incoming request to create a complete request for OpenRouter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionCreateParams {
    /// Messages for the conversation, including any prefix/suffix from the Agent.
    pub messages: Vec<crate::agent::completions::message::Message>,
    /// Provider preferences merged from request and Agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<super::Provider>,

    /// The model identifier from the Agent.
    pub model: String,
    /// Frequency penalty from Agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f64>,
    /// Logit bias from Agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<IndexMap<String, i64>>,
    /// Maximum completion tokens from Agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<u64>,
    /// Presence penalty from Agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f64>,
    /// Stop sequences from Agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<crate::agent::Stop>,
    /// Temperature from Agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    /// Top-p (nucleus sampling) from Agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f64>,
    /// Maximum tokens (legacy) from Agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u64>,
    /// Min-p sampling from Agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_p: Option<f64>,
    /// Reasoning configuration from Agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<crate::agent::Reasoning>,
    /// Repetition penalty from Agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repetition_penalty: Option<f64>,
    /// Top-a sampling from Agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_a: Option<f64>,
    /// Top-k sampling from Agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u64>,
    /// Verbosity setting from Agent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verbosity: Option<crate::agent::Verbosity>,

    /// Whether to include log probabilities from request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,
    /// Number of top log probabilities to return from request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u64>,
    /// Response format specification from request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<crate::agent::completions::request::ResponseFormat>,
    /// Random seed from request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    /// Tool choice configuration from request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<super::ToolChoice>,
    /// Available tools from request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<super::Tool>>,
    /// Whether to allow parallel tool calls from request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
    /// Prediction hints from request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prediction: Option<super::Prediction>,

    /// Always true for streaming requests.
    pub stream: bool,
    /// Stream options for usage inclusion.
    pub stream_options: super::StreamOptions,
    /// Usage reporting options.
    pub usage: super::Usage,
}

impl ChatCompletionCreateParams {
    /// Creates request parameters from an Agent and a chat completion request.
    ///
    /// Applies the Agent's prefix/suffix messages and decoding parameters.
    pub fn new(
        agent: &crate::agent::Agent,
        request: &crate::agent::completions::request::ChatCompletionCreateParams,
    ) -> Self {
        Self {
            messages: super::prompt::new(
                agent.base.prefix_messages.as_deref(),
                &request.messages,
                agent.base.suffix_messages.as_deref(),
            ),
            provider: super::provider::Provider::new(
                request.provider,
                agent.base.provider.as_ref(),
            ),
            model: agent.base.model.clone(),
            frequency_penalty: agent.base.frequency_penalty,
            logit_bias: agent.base.logit_bias.clone(),
            max_completion_tokens: agent.base.max_completion_tokens,
            presence_penalty: agent.base.presence_penalty,
            stop: agent.base.stop.clone(),
            temperature: agent.base.temperature,
            top_p: agent.base.top_p,
            max_tokens: agent.base.max_tokens,
            min_p: agent.base.min_p,
            reasoning: agent.base.reasoning,
            repetition_penalty: agent.base.repetition_penalty,
            top_a: agent.base.top_a,
            top_k: agent.base.top_k,
            verbosity: agent.base.verbosity,
            logprobs: if let Some(top_logprobs) = request.top_logprobs {
                Some(top_logprobs > 0)
            } else {
                None
            },
            top_logprobs: request.top_logprobs,
            response_format: request.response_format.clone(),
            seed: request.seed,
            tool_choice: None,
            tools: None,
            parallel_tool_calls: None,
            prediction: None,
            stream: true,
            stream_options: super::StreamOptions {
                include_usage: Some(true),
            },
            usage: super::Usage { include: true },
        }
    }

}
