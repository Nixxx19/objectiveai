use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaBashCodeExecutionToolResultBlockType {
    BashCodeExecutionToolResult,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaBashCodeExecutionToolResultBlock {
    pub content: BetaBashCodeExecutionToolResultBlockContent,
    pub tool_use_id: String,
    #[serde(rename = "type")]
    pub r#type: BetaBashCodeExecutionToolResultBlockType,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum BetaBashCodeExecutionToolResultBlockContent {
    Error(BetaBashCodeExecutionToolResultError),
    Result(BetaBashCodeExecutionResultBlock),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaBashCodeExecutionToolResultErrorType {
    BashCodeExecutionToolResultError,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaBashCodeExecutionToolResultError {
    pub error_code: BetaBashCodeExecutionToolResultErrorCode,
    #[serde(rename = "type")]
    pub r#type: BetaBashCodeExecutionToolResultErrorType,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaBashCodeExecutionToolResultErrorCode {
    InvalidToolInput,
    Unavailable,
    TooManyRequests,
    ExecutionTimeExceeded,
    OutputFileTooLarge,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaBashCodeExecutionResultBlockType {
    BashCodeExecutionResult,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaBashCodeExecutionResultBlock {
    pub content: Vec<super::BetaBashCodeExecutionOutputBlock>,
    pub return_code: i64,
    pub stderr: String,
    pub stdout: String,
    #[serde(rename = "type")]
    pub r#type: BetaBashCodeExecutionResultBlockType,
}
