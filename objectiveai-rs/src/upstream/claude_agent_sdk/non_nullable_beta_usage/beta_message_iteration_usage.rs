use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaMessageIterationUsageType {
    Message,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaMessageIterationUsage {
    pub cache_creation: Option<super::BetaCacheCreation>,
    pub cache_creation_input_tokens: f64,
    pub cache_read_input_tokens: f64,
    pub input_tokens: f64,
    pub output_tokens: f64,
    pub r#type: BetaMessageIterationUsageType,
}
