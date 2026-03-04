use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum BetaIterationsUsageItem {
    Message(super::BetaMessageIterationUsage),
    Compaction(super::BetaCompactionIterationUsage),
}

pub type BetaIterationsUsage = Vec<BetaIterationsUsageItem>;
