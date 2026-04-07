use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum BetaRawContentBlockDelta {
    Text(super::BetaTextDelta),
    InputJSON(super::BetaInputJSONDelta),
    Citations(super::BetaCitationsDelta),
    Thinking(super::BetaThinkingDelta),
    Signature(super::BetaSignatureDelta),
    Compaction(super::BetaCompactionContentBlockDelta),
}
