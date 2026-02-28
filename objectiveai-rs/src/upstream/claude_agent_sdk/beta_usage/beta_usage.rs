use serde::{Deserialize, Serialize};

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
    pub service_tier: Option<super::non_nullable_beta_usage::ServiceTier>,
    pub speed: Option<super::non_nullable_beta_usage::Speed>,
}
