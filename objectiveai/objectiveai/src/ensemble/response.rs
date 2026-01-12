use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListEnsemble {
    pub data: Vec<ListEnsembleItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListEnsembleItem {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetEnsemble {
    pub created: u64,
    #[serde(flatten)]
    pub inner: super::Ensemble,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataEnsemble {
    pub requests: u64,
    pub completion_tokens: u64,
    pub prompt_tokens: u64,
    pub total_cost: rust_decimal::Decimal,
}
