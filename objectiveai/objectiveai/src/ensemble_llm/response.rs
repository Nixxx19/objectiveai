use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListEnsembleLlm {
    pub data: Vec<ListEnsembleLlmItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListEnsembleLlmItem {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetEnsembleLlm {
    pub created: u64,
    #[serde(flatten)]
    pub inner: super::EnsembleLlm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataEnsembleLlm {
    pub requests: u64,
    pub completion_tokens: u64,
    pub prompt_tokens: u64,
    pub total_cost: rust_decimal::Decimal,
}
