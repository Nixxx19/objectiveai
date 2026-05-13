use serde::{Deserialize, Serialize};

/// Result of `laboratories executions create`. One item per agent
/// evaluated.
///
/// Wire: `{"type":"notification","laboratory":[...LabResultItem...]}`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Laboratory {
    pub laboratory: Vec<LabResultItem>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LabResultItem {
    pub agent: objectiveai_sdk::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional,
    pub score: Option<f64>,
    pub error: Option<objectiveai_sdk::error::ResponseError>,
}
