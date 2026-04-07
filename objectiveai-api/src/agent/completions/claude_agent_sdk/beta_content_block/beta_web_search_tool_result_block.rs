use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaWebSearchToolResultBlockType {
    WebSearchToolResult,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaWebSearchToolResultBlock {
    pub content: BetaWebSearchToolResultBlockContent,
    pub tool_use_id: String,
    #[serde(rename = "type")]
    pub r#type: BetaWebSearchToolResultBlockType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller: Option<super::BetaCaller>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum BetaWebSearchToolResultBlockContent {
    Error(BetaWebSearchToolResultError),
    Results(Vec<BetaWebSearchResultBlock>),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaWebSearchToolResultErrorType {
    WebSearchToolResultError,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaWebSearchToolResultError {
    pub error_code: BetaWebSearchToolResultErrorCode,
    #[serde(rename = "type")]
    pub r#type: BetaWebSearchToolResultErrorType,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaWebSearchToolResultErrorCode {
    InvalidToolInput,
    Unavailable,
    MaxUsesExceeded,
    TooManyRequests,
    QueryTooLong,
    RequestTooLarge,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaWebSearchResultBlockType {
    WebSearchResult,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaWebSearchResultBlock {
    pub encrypted_content: String,
    pub page_age: Option<String>,
    pub title: String,
    #[serde(rename = "type")]
    pub r#type: BetaWebSearchResultBlockType,
    pub url: String,
}
