use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaServerToolUseBlockType {
    ServerToolUse,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ServerToolName {
    WebSearch,
    WebFetch,
    CodeExecution,
    BashCodeExecution,
    TextEditorCodeExecution,
    ToolSearchToolRegex,
    ToolSearchToolBm25,
}

impl ServerToolName {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::WebSearch => "web_search",
            Self::WebFetch => "web_fetch",
            Self::CodeExecution => "code_execution",
            Self::BashCodeExecution => "bash_code_execution",
            Self::TextEditorCodeExecution => "text_editor_code_execution",
            Self::ToolSearchToolRegex => "tool_search_tool_regex",
            Self::ToolSearchToolBm25 => "tool_search_tool_bm25",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaServerToolUseBlock {
    pub id: String,
    pub input: indexmap::IndexMap<String, serde_json::Value>,
    pub name: ServerToolName,
    pub r#type: BetaServerToolUseBlockType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller: Option<super::BetaCaller>,
}
