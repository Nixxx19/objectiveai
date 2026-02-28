use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaWebFetchToolResultBlockType {
    WebFetchToolResult,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaWebFetchToolResultBlock {
    pub content: BetaWebFetchToolResultBlockContent,
    pub tool_use_id: String,
    #[serde(rename = "type")]
    pub r#type: BetaWebFetchToolResultBlockType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller: Option<super::BetaCaller>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum BetaWebFetchToolResultBlockContent {
    Error(BetaWebFetchToolResultErrorBlock),
    Result(BetaWebFetchBlock),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaWebFetchToolResultErrorBlockType {
    WebFetchToolResultError,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaWebFetchToolResultErrorBlock {
    pub error_code: BetaWebFetchToolResultErrorCode,
    #[serde(rename = "type")]
    pub r#type: BetaWebFetchToolResultErrorBlockType,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaWebFetchToolResultErrorCode {
    InvalidToolInput,
    UrlTooLong,
    UrlNotAllowed,
    UrlNotAccessible,
    UnsupportedContentType,
    TooManyRequests,
    MaxUsesExceeded,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaWebFetchBlockType {
    WebFetchResult,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaWebFetchBlock {
    pub content: super::BetaDocumentBlock,
    pub retrieved_at: Option<String>,
    #[serde(rename = "type")]
    pub r#type: BetaWebFetchBlockType,
    pub url: String,
}
