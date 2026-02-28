use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaRawContentBlockDeltaEventType {
    ContentBlockDelta,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaRawContentBlockDeltaEvent {
    pub delta: super::BetaRawContentBlockDelta,
    pub index: f64,
    pub r#type: BetaRawContentBlockDeltaEventType,
}
