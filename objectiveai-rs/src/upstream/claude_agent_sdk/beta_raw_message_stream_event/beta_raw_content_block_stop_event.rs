use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaRawContentBlockStopEventType {
    ContentBlockStop,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaRawContentBlockStopEvent {
    pub index: i64,
    pub r#type: BetaRawContentBlockStopEventType,
}
