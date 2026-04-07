use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaThinkingDeltaType {
    ThinkingDelta,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaThinkingDelta {
    pub thinking: String,
    pub r#type: BetaThinkingDeltaType,
}
