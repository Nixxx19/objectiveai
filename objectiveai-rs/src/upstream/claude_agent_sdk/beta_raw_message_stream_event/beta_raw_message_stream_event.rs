use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum BetaRawMessageStreamEvent {
    MessageStart(super::BetaRawMessageStartEvent),
    MessageDelta(super::BetaRawMessageDeltaEvent),
    MessageStop(super::BetaRawMessageStopEvent),
    ContentBlockStart(super::BetaRawContentBlockStartEvent),
    ContentBlockDelta(super::BetaRawContentBlockDeltaEvent),
    ContentBlockStop(super::BetaRawContentBlockStopEvent),
}
