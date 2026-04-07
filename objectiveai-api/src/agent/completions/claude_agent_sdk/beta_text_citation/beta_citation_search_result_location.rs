use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaCitationSearchResultLocationType {
    SearchResultLocation,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaCitationSearchResultLocation {
    pub cited_text: String,
    pub end_block_index: i64,
    pub search_result_index: i64,
    pub source: String,
    pub start_block_index: i64,
    pub title: Option<String>,
    pub r#type: BetaCitationSearchResultLocationType,
}
