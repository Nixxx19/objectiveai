use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Provider {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_collection: Option<ProviderDataCollection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zdr: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<ProviderSort>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_price: Option<ProviderMaxPrice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_min_throughput: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_max_latency: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_throughput: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_latency: Option<f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderDataCollection {
    Deny,
    Allow,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderSort {
    Price,
    Throughput,
    Latency,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ProviderMaxPrice {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<rust_decimal::Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion: Option<rust_decimal::Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<rust_decimal::Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<rust_decimal::Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request: Option<rust_decimal::Decimal>,
}
