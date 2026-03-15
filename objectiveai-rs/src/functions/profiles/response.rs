//! Profile listing and usage response types.

use crate::functions;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Response from listing profiles.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "functions.profiles.ListProfile")]
pub struct ListProfile {
    /// List of available profiles.
    pub data: Vec<ListProfileItem>,
}

/// A profile in a list response.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "functions.profiles.ListProfileItem")]
pub struct ListProfileItem {
    /// The remote source where the profile is hosted.
    pub remote: functions::Remote,
    /// Repository owner.
    pub owner: String,
    /// Repository name.
    pub repository: String,
    /// Git commit SHA.
    pub commit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "functions.profiles.GetProfile")]
pub struct GetProfile {
    pub remote: functions::Remote,
    pub owner: String,
    pub repository: String,
    pub commit: String,
    #[serde(flatten)]
    #[schemars(schema_with = "crate::flatten_schema::<functions::RemoteProfile>")]
    pub inner: functions::RemoteProfile,
}

/// Usage statistics for a profile.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "functions.profiles.UsageProfile")]
pub struct UsageProfile {
    /// Total number of requests made with this profile.
    pub requests: u64,
    /// Total completion tokens used.
    pub completion_tokens: u64,
    /// Total prompt tokens used.
    pub prompt_tokens: u64,
    /// Total cost incurred.
    #[schemars(with = "f64")]
    pub total_cost: rust_decimal::Decimal,
}
