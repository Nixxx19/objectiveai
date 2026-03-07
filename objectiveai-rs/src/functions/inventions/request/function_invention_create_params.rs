use crate::{agent, functions};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInventionCreateParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote: Option<functions::Remote>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github_token: Option<String>,
    pub state: functions::inventions::ParamsState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<agent::completions::request::Provider>,
    pub agent: agent::completions::request::Agent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agents: Option<Vec<agent::completions::request::Agent>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    /// Maximum number of retries per invention step.
    /// Each step is one agent completion (which itself may loop internally
    /// via tool calls). If the step's validation still fails after the
    /// agent loop ends, the step is retried up to this many times.
    /// Defaults to 3 if not specified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_step_retries: Option<u32>,
    /// Map from MCP server URL to authorization header value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mcp_server_authorization: Option<IndexMap<String, String>>,
}
