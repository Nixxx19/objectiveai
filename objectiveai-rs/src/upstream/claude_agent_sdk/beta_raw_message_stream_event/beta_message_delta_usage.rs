use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaMessageDeltaUsage {
    pub cache_creation_input_tokens: Option<f64>,
    pub cache_read_input_tokens: Option<f64>,
    pub input_tokens: Option<f64>,
    pub iterations: Option<super::super::beta_usage::BetaIterationsUsage>,
    pub output_tokens: f64,
    pub server_tool_use: Option<super::super::beta_usage::BetaServerToolUsage>,
}
