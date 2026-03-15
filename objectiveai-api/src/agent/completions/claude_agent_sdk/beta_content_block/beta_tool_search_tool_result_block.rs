use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaToolSearchToolResultBlockType {
    ToolSearchToolResult,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaToolSearchToolResultErrorCode {
    InvalidToolInput,
    Unavailable,
    TooManyRequests,
    ExecutionTimeExceeded,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaToolSearchToolResultErrorType {
    ToolSearchToolResultError,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaToolSearchToolResultError {
    pub error_code: BetaToolSearchToolResultErrorCode,
    pub error_message: Option<String>,
    pub r#type: BetaToolSearchToolResultErrorType,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaToolSearchToolSearchResultBlockType {
    ToolSearchToolSearchResult,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaToolReferenceBlockType {
    ToolReference,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaToolReferenceBlock {
    pub tool_name: String,
    pub r#type: BetaToolReferenceBlockType,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaToolSearchToolSearchResultBlock {
    pub tool_references: Vec<BetaToolReferenceBlock>,
    pub r#type: BetaToolSearchToolSearchResultBlockType,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum BetaToolSearchToolResultContent {
    Error(BetaToolSearchToolResultError),
    SearchResult(BetaToolSearchToolSearchResultBlock),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaToolSearchToolResultBlock {
    pub content: BetaToolSearchToolResultContent,
    pub tool_use_id: String,
    pub r#type: BetaToolSearchToolResultBlockType,
}
