use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaRawContentBlockStartEventType {
    ContentBlockStart,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaRawContentBlockStartEvent {
    pub content_block: super::super::beta_content_block::BetaContentBlock,
    pub index: f64,
    pub r#type: BetaRawContentBlockStartEventType,
}
