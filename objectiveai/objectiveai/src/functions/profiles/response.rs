use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListProfile {
    pub data: Vec<ListProfileItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListProfileItem {
    pub owner: String,
    pub repository: String,
    pub commit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageProfile {
    pub requests: u64,
    pub completion_tokens: u64,
    pub prompt_tokens: u64,
    pub total_cost: rust_decimal::Decimal,
}
