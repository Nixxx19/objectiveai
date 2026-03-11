//! Provider preferences for agent completion requests.

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Provider routing and selection preferences.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "agent.completions.request.Provider")]
pub struct Provider {
    /// Whether to allow providers to collect data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_collection: Option<ProviderDataCollection>,
    /// Whether to use zero data retention providers only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zdr: Option<bool>,
    /// How to sort/prioritize providers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<ProviderSort>,
    /// Maximum price constraints.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_price: Option<ProviderMaxPrice>,
    /// Preferred minimum throughput (tokens/second).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_min_throughput: Option<f64>,
    /// Preferred maximum latency (seconds).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_max_latency: Option<f64>,
    /// Hard minimum throughput requirement (tokens/second).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_throughput: Option<f64>,
    /// Hard maximum latency requirement (seconds).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_latency: Option<f64>,
}

/// Data collection policy for providers.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(rename = "agent.completions.request.ProviderDataCollection")]
pub enum ProviderDataCollection {
    /// Do not allow data collection.
    Deny,
    /// Allow data collection.
    Allow,
}

/// How to sort/prioritize providers.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(rename = "agent.completions.request.ProviderSort")]
pub enum ProviderSort {
    /// Prioritize by price (cheapest first).
    Price,
    /// Prioritize by throughput (fastest first).
    Throughput,
    /// Prioritize by latency (lowest first).
    Latency,
}

/// Maximum price constraints per token type.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "agent.completions.request.ProviderMaxPrice")]
pub struct ProviderMaxPrice {
    /// Maximum price per prompt token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<rust_decimal::Decimal>,
    /// Maximum price per completion token.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion: Option<rust_decimal::Decimal>,
    /// Maximum price per image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<rust_decimal::Decimal>,
    /// Maximum price per audio second.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<rust_decimal::Decimal>,
    /// Maximum price per request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<rust_decimal::Decimal>,
}
