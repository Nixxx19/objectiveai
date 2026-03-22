use crate::{agent, functions};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Parameters for creating a function execution.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "functions.executions.request.FunctionExecutionCreateParams")]
pub struct FunctionExecutionCreateParams {
    /// The function to execute (inline definition or remote path).
    pub function: functions::FullInlineFunctionOrRemoteCommitOptional,
    /// The profile to use (inline definition or remote path).
    pub profile: functions::InlineProfileOrRemoteCommitOptional,

    // --- Caching and retry options ---
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_cache: Option<bool>,

    // --- Reasoning configuration ---
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<super::Reasoning>,

    // --- Core configuration ---
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<super::Strategy>,
    pub input: functions::expression::InputValue,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<agent::completions::request::Provider>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}
