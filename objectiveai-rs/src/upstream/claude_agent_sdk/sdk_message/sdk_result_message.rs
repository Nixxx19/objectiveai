use serde::{Deserialize, Serialize};
use indexmap::IndexMap;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum SDKResultMessage {
    Success(SDKResultSuccess),
    Error(SDKResultError),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SDKResultSuccess {
    #[serde(rename = "type")]
    pub r#type: String,
    pub subtype: String,
    pub duration_ms: f64,
    pub duration_api_ms: f64,
    pub is_error: bool,
    pub num_turns: f64,
    pub result: String,
    pub stop_reason: Option<String>,
    pub total_cost_usd: f64,
    pub usage: super::super::beta_usage::NonNullableBetaUsage,
    #[serde(rename = "modelUsage")]
    pub model_usage: IndexMap<String, ModelUsage>,
    pub permission_denials: Vec<SDKPermissionDenial>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub structured_output: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fast_mode_state: Option<super::FastModeState>,
    pub uuid: String,
    pub session_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SDKResultError {
    #[serde(rename = "type")]
    pub r#type: String,
    pub subtype: SDKResultErrorSubtype,
    pub duration_ms: f64,
    pub duration_api_ms: f64,
    pub is_error: bool,
    pub num_turns: f64,
    pub stop_reason: Option<String>,
    pub total_cost_usd: f64,
    pub usage: super::super::beta_usage::NonNullableBetaUsage,
    #[serde(rename = "modelUsage")]
    pub model_usage: IndexMap<String, ModelUsage>,
    pub permission_denials: Vec<SDKPermissionDenial>,
    pub errors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fast_mode_state: Option<super::FastModeState>,
    pub uuid: String,
    pub session_id: String,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SDKResultErrorSubtype {
    ErrorDuringExecution,
    ErrorMaxTurns,
    ErrorMaxBudgetUsd,
    ErrorMaxStructuredOutputRetries,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SDKPermissionDenial {
    pub tool_name: String,
    pub tool_use_id: String,
    pub tool_input: indexmap::IndexMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModelUsage {
    pub input_tokens: f64,
    pub output_tokens: f64,
    pub cache_read_input_tokens: f64,
    pub cache_creation_input_tokens: f64,
    pub web_search_requests: f64,
    pub cost_usd: f64,
    pub context_window: f64,
    pub max_output_tokens: f64,
}
