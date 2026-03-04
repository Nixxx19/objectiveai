use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaCodeExecutionToolResultBlockType {
    CodeExecutionToolResult,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaCodeExecutionToolResultBlock {
    pub content: BetaCodeExecutionToolResultBlockContent,
    pub tool_use_id: String,
    #[serde(rename = "type")]
    pub r#type: BetaCodeExecutionToolResultBlockType,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum BetaCodeExecutionToolResultBlockContent {
    Error(BetaCodeExecutionToolResultError),
    Result(BetaCodeExecutionResultBlock),
    EncryptedResult(BetaEncryptedCodeExecutionResultBlock),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaCodeExecutionToolResultErrorType {
    CodeExecutionToolResultError,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaCodeExecutionToolResultError {
    pub error_code: BetaCodeExecutionToolResultErrorCode,
    #[serde(rename = "type")]
    pub r#type: BetaCodeExecutionToolResultErrorType,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaCodeExecutionToolResultErrorCode {
    InvalidToolInput,
    Unavailable,
    TooManyRequests,
    ExecutionTimeExceeded,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaCodeExecutionResultBlockType {
    CodeExecutionResult,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaCodeExecutionResultBlock {
    pub content: Vec<super::BetaCodeExecutionOutputBlock>,
    pub return_code: i64,
    pub stderr: String,
    pub stdout: String,
    #[serde(rename = "type")]
    pub r#type: BetaCodeExecutionResultBlockType,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaEncryptedCodeExecutionResultBlockType {
    EncryptedCodeExecutionResult,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaEncryptedCodeExecutionResultBlock {
    pub content: Vec<super::BetaCodeExecutionOutputBlock>,
    pub encrypted_stdout: String,
    pub return_code: i64,
    pub stderr: String,
    #[serde(rename = "type")]
    pub r#type: BetaEncryptedCodeExecutionResultBlockType,
}
