use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BashCodeExecutionToolResultBlockParamType {
    BashCodeExecutionToolResult,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BashCodeExecutionToolResultErrorCode {
    InvalidToolInput,
    Unavailable,
    TooManyRequests,
    ExecutionTimeExceeded,
    OutputFileTooLarge,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BashCodeExecutionToolResultErrorParamType {
    BashCodeExecutionToolResultError,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BashCodeExecutionToolResultErrorParam {
    pub error_code: BashCodeExecutionToolResultErrorCode,
    pub r#type: BashCodeExecutionToolResultErrorParamType,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BashCodeExecutionOutputBlockParamType {
    BashCodeExecutionOutput,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BashCodeExecutionOutputBlockParam {
    pub file_id: String,
    pub r#type: BashCodeExecutionOutputBlockParamType,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BashCodeExecutionResultBlockParamType {
    BashCodeExecutionResult,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BashCodeExecutionResultBlockParam {
    pub content: Vec<BashCodeExecutionOutputBlockParam>,
    pub return_code: i64,
    pub stderr: String,
    pub stdout: String,
    pub r#type: BashCodeExecutionResultBlockParamType,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum BashCodeExecutionToolResultBlockParamContent {
    Error(BashCodeExecutionToolResultErrorParam),
    Result(BashCodeExecutionResultBlockParam),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BashCodeExecutionToolResultBlockParam {
    pub content: BashCodeExecutionToolResultBlockParamContent,
    pub tool_use_id: String,
    pub r#type: BashCodeExecutionToolResultBlockParamType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<super::CacheControlEphemeral>,
}
