use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ServiceTier {
    Standard,
    Priority,
    Batch,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Speed {
    Standard,
    Fast,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaUsage {
    pub cache_creation: Option<super::BetaCacheCreation>,
    pub cache_creation_input_tokens: Option<f64>,
    pub cache_read_input_tokens: Option<f64>,
    pub inference_geo: Option<String>,
    pub input_tokens: f64,
    pub iterations: Option<super::BetaIterationsUsage>,
    pub output_tokens: f64,
    pub server_tool_use: Option<super::BetaServerToolUsage>,
    pub service_tier: Option<ServiceTier>,
    pub speed: Option<Speed>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NonNullableBetaUsage {
    pub cache_creation: super::BetaCacheCreation,
    pub cache_creation_input_tokens: f64,
    pub cache_read_input_tokens: f64,
    pub inference_geo: String,
    pub input_tokens: f64,
    pub iterations: super::BetaIterationsUsage,
    pub output_tokens: f64,
    pub server_tool_use: super::BetaServerToolUsage,
    pub service_tier: ServiceTier,
    pub speed: Speed,
}
