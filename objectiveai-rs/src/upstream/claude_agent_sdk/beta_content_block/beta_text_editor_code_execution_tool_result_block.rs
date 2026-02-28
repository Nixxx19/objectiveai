use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaTextEditorCodeExecutionToolResultBlockType {
    TextEditorCodeExecutionToolResult,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaTextEditorCodeExecutionToolResultBlock {
    pub content: BetaTextEditorCodeExecutionToolResultBlockContent,
    pub tool_use_id: String,
    #[serde(rename = "type")]
    pub r#type: BetaTextEditorCodeExecutionToolResultBlockType,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum BetaTextEditorCodeExecutionToolResultBlockContent {
    Error(BetaTextEditorCodeExecutionToolResultError),
    ViewResult(BetaTextEditorCodeExecutionViewResultBlock),
    CreateResult(BetaTextEditorCodeExecutionCreateResultBlock),
    StrReplaceResult(BetaTextEditorCodeExecutionStrReplaceResultBlock),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaTextEditorCodeExecutionToolResultErrorType {
    TextEditorCodeExecutionToolResultError,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaTextEditorCodeExecutionToolResultError {
    pub error_code: BetaTextEditorCodeExecutionToolResultErrorCode,
    pub error_message: Option<String>,
    #[serde(rename = "type")]
    pub r#type: BetaTextEditorCodeExecutionToolResultErrorType,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaTextEditorCodeExecutionToolResultErrorCode {
    InvalidToolInput,
    Unavailable,
    TooManyRequests,
    ExecutionTimeExceeded,
    FileNotFound,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaTextEditorCodeExecutionViewResultBlockType {
    TextEditorCodeExecutionViewResult,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaTextEditorCodeExecutionViewResultBlockFileType {
    Text,
    Image,
    Pdf,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaTextEditorCodeExecutionViewResultBlock {
    pub content: String,
    pub file_type: BetaTextEditorCodeExecutionViewResultBlockFileType,
    pub num_lines: Option<f64>,
    pub start_line: Option<f64>,
    pub total_lines: Option<f64>,
    #[serde(rename = "type")]
    pub r#type: BetaTextEditorCodeExecutionViewResultBlockType,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaTextEditorCodeExecutionCreateResultBlockType {
    TextEditorCodeExecutionCreateResult,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaTextEditorCodeExecutionCreateResultBlock {
    pub is_file_update: bool,
    #[serde(rename = "type")]
    pub r#type: BetaTextEditorCodeExecutionCreateResultBlockType,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaTextEditorCodeExecutionStrReplaceResultBlockType {
    TextEditorCodeExecutionStrReplaceResult,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaTextEditorCodeExecutionStrReplaceResultBlock {
    pub lines: Option<Vec<String>>,
    pub new_lines: Option<f64>,
    pub new_start: Option<f64>,
    pub old_lines: Option<f64>,
    pub old_start: Option<f64>,
    #[serde(rename = "type")]
    pub r#type: BetaTextEditorCodeExecutionStrReplaceResultBlockType,
}
