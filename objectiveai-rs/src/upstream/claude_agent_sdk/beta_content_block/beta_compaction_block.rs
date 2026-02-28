use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaCompactionBlockType {
    Compaction,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaCompactionBlock {
    pub content: Option<String>,
    pub r#type: BetaCompactionBlockType,
}
