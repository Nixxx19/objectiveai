use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaInputJSONDeltaType {
    InputJsonDelta,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaInputJSONDelta {
    pub partial_json: String,
    pub r#type: BetaInputJSONDeltaType,
}
