//! Agent completion request parameters.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Parameters for creating a agent completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCompletionCreateParams {
    /// The conversation messages.
    pub messages: Vec<super::super::message::Message>,
    /// Provider routing preferences.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<super::Provider>,
    /// The agent to use (inline Agent or stored ID).
    pub agent: super::Agent,
    /// Alternative agents to try if the primary agent fails.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agents: Option<Vec<super::Agent>>,
    /// Output format constraints (text, JSON, or JSON schema).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<super::ResponseFormat>,
    /// Random seed for deterministic generation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    /// Whether to stream the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

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
