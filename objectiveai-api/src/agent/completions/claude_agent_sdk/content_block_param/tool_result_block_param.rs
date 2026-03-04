use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolResultBlockParamType {
    ToolResult,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ToolResultContentBlockParam {
    Text(super::TextBlockParam),
    Image(super::ImageBlockParam),
    SearchResult(super::SearchResultBlockParam),
    Document(super::DocumentBlockParam),
    ToolReference(super::ToolReferenceBlockParam),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ToolResultBlockParamContent {
    String(String),
    Blocks(Vec<ToolResultContentBlockParam>),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ToolResultBlockParam {
    pub tool_use_id: String,
    pub r#type: ToolResultBlockParamType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_control: Option<super::CacheControlEphemeral>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<ToolResultBlockParamContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}
