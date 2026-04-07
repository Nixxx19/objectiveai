use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaThinkingBlockType {
    Thinking,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaThinkingBlock {
    pub signature: String,
    pub thinking: String,
    pub r#type: BetaThinkingBlockType,
}
