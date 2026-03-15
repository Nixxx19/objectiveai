use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaCitationsWebSearchResultLocationType {
    WebSearchResultLocation,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaCitationsWebSearchResultLocation {
    pub cited_text: String,
    pub encrypted_index: String,
    pub title: Option<String>,
    pub r#type: BetaCitationsWebSearchResultLocationType,
    pub url: String,
}
