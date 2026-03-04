use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaSignatureDeltaType {
    SignatureDelta,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaSignatureDelta {
    pub signature: String,
    pub r#type: BetaSignatureDeltaType,
}
