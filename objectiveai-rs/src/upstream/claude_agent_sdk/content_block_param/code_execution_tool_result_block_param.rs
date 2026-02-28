use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodeExecutionToolResultBlockParamType {
    CodeExecutionToolResult,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodeExecutionToolResultErrorCode {
    InvalidToolInput,
    Unavailable,
    TooManyRequests,
    ExecutionTimeExceeded,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodeExecutionToolResultErrorParamType {
    CodeExecutionToolResultError,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CodeExecutionToolResultErrorParam {
    pub error_code: CodeExecutionToolResultErrorCode,
    pub r#type: CodeExecutionToolResultErrorParamType,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodeExecutionOutputBlockParamType {
    CodeExecutionOutput,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CodeExecutionOutputBlockParam {
    pub file_id: String,
    pub r#type: CodeExecutionOutputBlockParamType,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CodeExecutionResultBlockParamType {
    CodeExecutionResult,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CodeExecutionResultBlockParam {
    pub content: Vec<CodeExecutionOutputBlockParam>,
    pub return_code: f64,
    pub stderr: String,
    pub stdout: String,
    pub r#type: CodeExecutionResultBlockParamType,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EncryptedCodeExecutionResultBlockParamType {
    EncryptedCodeExecutionResult,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct EncryptedCodeExecutionResultBlockParam {
    pub content: Vec<CodeExecutionOutputBlockParam>,
    pub encrypted_stdout: String,
    pub return_code: f64,
    pub stderr: String,
    pub r#type: EncryptedCodeExecutionResultBlockParamType,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum CodeExecutionToolResultBlockParamContent {
    Error(CodeExecutionToolResultErrorParam),
    Result(CodeExecutionResultBlockParam),
    EncryptedResult(EncryptedCodeExecutionResultBlockParam),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CodeExecutionToolResultBlockParam {
    pub content: CodeExecutionToolResultBlockParamContent,
    pub tool_use_id: String,
    pub r#type: CodeExecutionToolResultBlockParamType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<super::CacheControlEphemeral>,
}
