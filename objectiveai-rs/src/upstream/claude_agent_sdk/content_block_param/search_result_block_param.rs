use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SearchResultBlockParamType {
    SearchResult,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SearchResultBlockParam {
    pub content: Vec<super::TextBlockParam>,
    pub source: String,
    pub title: String,
    pub r#type: SearchResultBlockParamType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<super::CacheControlEphemeral>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citations: Option<super::CitationsConfigParam>,
}
