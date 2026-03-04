use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaRawMessageStartEventType {
    MessageStart,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaRawMessageStartEvent {
    pub message: super::super::beta_message::BetaMessage,
    pub r#type: BetaRawMessageStartEventType,
}
