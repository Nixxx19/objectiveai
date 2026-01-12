use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListFunction {
    pub data: Vec<ListFunctionItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListFunctionItem {
    pub owner: String,
    pub repository: String,
    pub commit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataFunction {
    pub requests: u64,
    pub completion_tokens: u64,
    pub prompt_tokens: u64,
    pub total_cost: rust_decimal::Decimal,
}
