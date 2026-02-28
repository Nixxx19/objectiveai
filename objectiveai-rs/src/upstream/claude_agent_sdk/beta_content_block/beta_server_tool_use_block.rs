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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaServerToolUseBlock {
    pub id: String,
    pub input: indexmap::IndexMap<String, serde_json::Value>,
    pub name: ServerToolName,
    pub r#type: BetaServerToolUseBlockType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caller: Option<serde_json::Value>,
}
