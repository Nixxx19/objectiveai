use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaRedactedThinkingBlockType {
    RedactedThinking,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaRedactedThinkingBlock {
    pub data: String,
    pub r#type: BetaRedactedThinkingBlockType,
}
