use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaRawMessageDeltaEventType {
    MessageDelta,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaRawMessageDeltaEventDelta {
    pub container: Option<super::super::beta_message::BetaContainer>,
    pub stop_reason: Option<super::super::beta_message::BetaStopReason>,
    pub stop_sequence: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaRawMessageDeltaEvent {
    pub context_management: Option<super::super::beta_message::BetaContextManagementResponse>,
    pub delta: BetaRawMessageDeltaEventDelta,
    pub r#type: BetaRawMessageDeltaEventType,
    pub usage: super::BetaMessageDeltaUsage,
}
