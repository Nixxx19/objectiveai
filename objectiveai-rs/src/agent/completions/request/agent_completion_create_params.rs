//! Agent completion request parameters.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Parameters for creating a agent completion.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "agent.completions.request.AgentCompletionCreateParams")]
pub struct AgentCompletionCreateParams {
    /// The conversation messages.
    pub messages: Vec<super::super::message::Message>,
    /// Provider routing preferences.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<super::Provider>,
    /// The agent to use (inline Agent or stored ID).
    pub agent: crate::agent::InlineAgentBaseWithFallbacksOrRemote,
    /// Output format constraints (text, JSON, or JSON schema).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<super::ResponseFormatParam>,
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

}
