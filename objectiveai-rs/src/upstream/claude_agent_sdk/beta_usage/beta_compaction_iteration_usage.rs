use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaCompactionIterationUsageType {
    Compaction,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaCompactionIterationUsage {
    pub cache_creation: Option<super::BetaCacheCreation>,
    pub cache_creation_input_tokens: i64,
    pub cache_read_input_tokens: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub r#type: BetaCompactionIterationUsageType,
}
