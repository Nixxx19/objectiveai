use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WebFetchToolResultBlockParamType {
    WebFetchToolResult,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WebFetchToolResultErrorCode {
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
pub enum WebFetchToolResultErrorBlockParamType {
    WebFetchToolResultError,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WebFetchToolResultErrorBlockParam {
    pub error_code: WebFetchToolResultErrorCode,
    pub r#type: WebFetchToolResultErrorBlockParamType,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WebFetchBlockParamType {
    WebFetchResult,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WebFetchBlockParam {
    pub content: super::DocumentBlockParam,
    pub r#type: WebFetchBlockParamType,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retrieved_at: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum WebFetchToolResultBlockParamContent {
    Error(WebFetchToolResultErrorBlockParam),
    Result(WebFetchBlockParam),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct WebFetchToolResultBlockParam {
    pub content: WebFetchToolResultBlockParamContent,
    pub tool_use_id: String,
    pub r#type: WebFetchToolResultBlockParamType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<super::CacheControlEphemeral>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller: Option<super::Caller>,
}
