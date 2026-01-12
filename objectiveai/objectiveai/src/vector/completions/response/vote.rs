use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub model: String,
    pub ensemble_index: u64,
    pub flat_ensemble_index: u64,
    pub vote: Vec<rust_decimal::Decimal>,
    pub weight: rust_decimal::Decimal,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retry: Option<bool>,
    #[serde(skip)]
    pub completion_index: Option<u64>,
}
