use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaCacheCreation {
    pub ephemeral_1h_input_tokens: f64,
    pub ephemeral_5m_input_tokens: f64,
}
