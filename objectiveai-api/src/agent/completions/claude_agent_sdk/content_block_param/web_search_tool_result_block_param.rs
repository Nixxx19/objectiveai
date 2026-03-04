use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WebSearchToolResultBlockParamType {
    WebSearchToolResult,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WebSearchToolResultErrorCode {
    InvalidToolInput,
    Unavailable,
    MaxUsesExceeded,
    TooManyRequests,
    QueryTooLong,
    RequestTooLarge,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WebSearchToolRequestErrorType {
    WebSearchToolResultError,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WebSearchToolRequestError {
    pub error_code: WebSearchToolResultErrorCode,
    pub r#type: WebSearchToolRequestErrorType,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WebSearchResultBlockParamType {
    WebSearchResult,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WebSearchResultBlockParam {
    pub encrypted_content: String,
    pub title: String,
    pub r#type: WebSearchResultBlockParamType,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_age: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum WebSearchToolResultBlockParamContent {
    Results(Vec<WebSearchResultBlockParam>),
    Error(WebSearchToolRequestError),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WebSearchToolResultBlockParam {
    pub content: WebSearchToolResultBlockParamContent,
    pub tool_use_id: String,
    pub r#type: WebSearchToolResultBlockParamType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<super::CacheControlEphemeral>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller: Option<super::Caller>,
}
