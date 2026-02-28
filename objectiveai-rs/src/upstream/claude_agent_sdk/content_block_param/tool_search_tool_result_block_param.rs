use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolSearchToolResultBlockParamType {
    ToolSearchToolResult,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolSearchToolResultErrorCode {
    InvalidToolInput,
    Unavailable,
    TooManyRequests,
    ExecutionTimeExceeded,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolSearchToolResultErrorParamType {
    ToolSearchToolResultError,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ToolSearchToolResultErrorParam {
    pub error_code: ToolSearchToolResultErrorCode,
    pub r#type: ToolSearchToolResultErrorParamType,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolSearchToolSearchResultBlockParamType {
    ToolSearchToolSearchResult,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ToolSearchToolSearchResultBlockParam {
    pub tool_references: Vec<super::ToolReferenceBlockParam>,
    pub r#type: ToolSearchToolSearchResultBlockParamType,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ToolSearchToolResultBlockParamContent {
    Error(ToolSearchToolResultErrorParam),
    SearchResult(ToolSearchToolSearchResultBlockParam),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ToolSearchToolResultBlockParam {
    pub content: ToolSearchToolResultBlockParamContent,
    pub tool_use_id: String,
    pub r#type: ToolSearchToolResultBlockParamType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<super::CacheControlEphemeral>,
}
