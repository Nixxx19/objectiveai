use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaCompactionContentBlockDeltaType {
    CompactionDelta,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaCompactionContentBlockDelta {
    pub content: Option<String>,
    pub r#type: BetaCompactionContentBlockDeltaType,
}
