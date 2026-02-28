use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TextEditorCodeExecutionToolResultBlockParamType {
    TextEditorCodeExecutionToolResult,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TextEditorCodeExecutionToolResultErrorCode {
    InvalidToolInput,
    Unavailable,
    TooManyRequests,
    ExecutionTimeExceeded,
    FileNotFound,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TextEditorCodeExecutionToolResultErrorParamType {
    TextEditorCodeExecutionToolResultError,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TextEditorCodeExecutionToolResultErrorParam {
    pub error_code: TextEditorCodeExecutionToolResultErrorCode,
    pub r#type: TextEditorCodeExecutionToolResultErrorParamType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TextEditorCodeExecutionViewResultBlockParamFileType {
    Text,
    Image,
    Pdf,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TextEditorCodeExecutionViewResultBlockParamType {
    TextEditorCodeExecutionViewResult,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TextEditorCodeExecutionViewResultBlockParam {
    pub content: String,
    pub file_type: TextEditorCodeExecutionViewResultBlockParamFileType,
    pub r#type: TextEditorCodeExecutionViewResultBlockParamType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_lines: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_line: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_lines: Option<i64>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TextEditorCodeExecutionCreateResultBlockParamType {
    TextEditorCodeExecutionCreateResult,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TextEditorCodeExecutionCreateResultBlockParam {
    pub is_file_update: bool,
    pub r#type: TextEditorCodeExecutionCreateResultBlockParamType,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TextEditorCodeExecutionStrReplaceResultBlockParamType {
    TextEditorCodeExecutionStrReplaceResult,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TextEditorCodeExecutionStrReplaceResultBlockParam {
    pub r#type: TextEditorCodeExecutionStrReplaceResultBlockParamType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lines: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_lines: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_start: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_lines: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_start: Option<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum TextEditorCodeExecutionToolResultBlockParamContent {
    Error(TextEditorCodeExecutionToolResultErrorParam),
    ViewResult(TextEditorCodeExecutionViewResultBlockParam),
    CreateResult(TextEditorCodeExecutionCreateResultBlockParam),
    StrReplaceResult(TextEditorCodeExecutionStrReplaceResultBlockParam),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TextEditorCodeExecutionToolResultBlockParam {
    pub content: TextEditorCodeExecutionToolResultBlockParamContent,
    pub tool_use_id: String,
    pub r#type: TextEditorCodeExecutionToolResultBlockParamType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<super::CacheControlEphemeral>,
}
