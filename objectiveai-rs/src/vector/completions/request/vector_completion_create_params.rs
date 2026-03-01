//! Vector completion request parameters.

use crate::agent;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Parameters for creating a vector completion.
///
/// Vector completions run multiple agent completions (one per LLM in the
/// ensemble), force each to vote for one of the predefined responses, and
/// combine votes using the provided profile weights to produce final scores.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorCompletionCreateParams {
    // --- Caching and retry options ---
    /// If present, reuses votes from a previous request with this ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry: Option<String>,
    /// If true, uses cached votes when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_cache: Option<bool>,
    /// If true, remaining votes are generated randomly (for testing/simulation).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_rng: Option<bool>,

    // --- Core configuration ---
    /// The conversation messages (the prompt).
    pub messages: Vec<agent::completions::message::Message>,
    /// Provider routing preferences.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<agent::completions::request::Provider>,
    /// The Ensemble of agents to use.
    pub ensemble: super::Ensemble,
    /// The profile weights for each agent in the ensemble.
    ///
    /// Must have the same length as the total agent count in the ensemble.
    /// Can be either:
    /// - A vector of decimals (legacy representation), or
    /// - A vector of objects with `weight` and optional `invert` fields.
    pub profile: super::Profile,
    /// Random seed for deterministic results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    /// Whether to stream the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    /// The possible responses the LLMs can vote for.
    pub responses: Vec<agent::completions::message::RichContent>,

    // --- MCP server authorization ---
    /// Map from MCP server URL to authorization header value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_server_authorization: Option<IndexMap<String, String>>,

    // --- Retry configuration ---
    /// Maximum elapsed time (ms) for exponential backoff retries.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backoff_max_elapsed_time: Option<u64>,
    /// Timeout (ms) for receiving the first chunk of a streaming response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_chunk_timeout: Option<u64>,
    /// Timeout (ms) between subsequent chunks of a streaming response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub other_chunk_timeout: Option<u64>,
}
