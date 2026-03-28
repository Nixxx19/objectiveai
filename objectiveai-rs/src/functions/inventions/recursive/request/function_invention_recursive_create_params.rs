use crate::{agent, functions};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "functions.inventions.recursive.request.FunctionInventionRecursiveCreateParams")]
pub struct FunctionInventionRecursiveCreateParams {
    pub remote: crate::Remote,
    pub name: String,
    pub state: functions::inventions::ParamsStateOrRemoteCommitOptional,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<agent::completions::request::Provider>,
    pub agent: agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional,
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
}
