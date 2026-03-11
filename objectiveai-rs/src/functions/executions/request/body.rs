//! Request body types for function executions.

use crate::{agent, functions};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Request body for inline Function with inline Profile.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "functions.executions.request.FunctionInlineProfileInlineRequestBody")]
pub struct FunctionInlineProfileInlineRequestBody {
    /// The inline Function definition.
    pub function: functions::InlineFunction,
    /// The inline Profile definition.
    pub profile: functions::InlineProfile,
    /// Common execution parameters.
    #[serde(flatten)]
    pub base: FunctionRemoteProfileRemoteRequestBody,
}

/// Request body for inline Function with remote Profile.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "functions.executions.request.FunctionInlineProfileRemoteRequestBody")]
pub struct FunctionInlineProfileRemoteRequestBody {
    /// The inline Function definition.
    pub function: functions::InlineFunction,
    /// Common execution parameters.
    #[serde(flatten)]
    pub base: FunctionRemoteProfileRemoteRequestBody,
}

/// Request body for remote Function with inline Profile.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "functions.executions.request.FunctionRemoteProfileInlineRequestBody")]
pub struct FunctionRemoteProfileInlineRequestBody {
    /// The inline Profile definition.
    pub profile: functions::InlineProfile,
    /// Common execution parameters.
    #[serde(flatten)]
    pub base: FunctionRemoteProfileRemoteRequestBody,
}

/// Base request body with common execution parameters.
///
/// Used directly for remote Function + remote Profile, or flattened into
/// other request body types.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "functions.executions.request.FunctionRemoteProfileRemoteRequestBody")]
pub struct FunctionRemoteProfileRemoteRequestBody {
    // --- Caching and retry options ---
    /// If present, reuses votes from a previous execution with this token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry_token: Option<String>,
    /// If true, uses cached votes when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_cache: Option<bool>,

    // --- Reasoning configuration ---
    /// Reasoning summary configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<super::Reasoning>,

    // --- Core configuration ---
    /// Execution strategy.
    /// Defaults to `Default` strategy if not specified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<super::Strategy>,
    /// The input data to pass to the Function.
    pub input: functions::expression::Input,
    /// Provider routing preferences.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<agent::completions::request::Provider>,
    /// Random seed for deterministic results.
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
