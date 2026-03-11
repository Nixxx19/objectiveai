//! Request types for function listing endpoints.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Query parameters for the list functions endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "functions.ListFunctionsQueryParameters")]
pub struct ListFunctionsQueryParameters {
    /// Optional source filter for listing functions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<ListFunctionsSource>,
}

/// Source filter for listing functions.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "functions.ListFunctionsSource")]
#[serde(rename_all = "snake_case")]
pub enum ListFunctionsSource {
    All,
    Mock,
    Filesystem,
    Objectiveai,
}

impl ListFunctionsSource {
    pub fn as_str(&self) -> &str {
        match self {
            ListFunctionsSource::All => "all",
            ListFunctionsSource::Mock => "mock",
            ListFunctionsSource::Filesystem => "filesystem",
            ListFunctionsSource::Objectiveai => "objectiveai",
        }
    }
}

/// Query parameters for the list function-profile pairs endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "functions.ListFunctionProfilePairsQueryParameters")]
pub struct ListFunctionProfilePairsQueryParameters {
    /// Optional source filter for listing function-profile pairs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<ListFunctionProfilePairsSource>,
}

/// Source filter for listing function-profile pairs.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "functions.ListFunctionProfilePairsSource")]
#[serde(rename_all = "snake_case")]
pub enum ListFunctionProfilePairsSource {
    Objectiveai,
}

impl ListFunctionProfilePairsSource {
    pub fn as_str(&self) -> &str {
        match self {
            ListFunctionProfilePairsSource::Objectiveai => "objectiveai",
        }
    }
}
