use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ContentBlockParam {
    Text(super::TextBlockParam),
    Image(super::ImageBlockParam),
    Document(super::DocumentBlockParam),
    SearchResult(super::SearchResultBlockParam),
    Thinking(super::ThinkingBlockParam),
    RedactedThinking(super::RedactedThinkingBlockParam),
    ToolUse(super::ToolUseBlockParam),
    ToolResult(super::ToolResultBlockParam),
    ServerToolUse(super::ServerToolUseBlockParam),
    WebSearchToolResult(super::WebSearchToolResultBlockParam),
    WebFetchToolResult(super::WebFetchToolResultBlockParam),
    CodeExecutionToolResult(super::CodeExecutionToolResultBlockParam),
    BashCodeExecutionToolResult(super::BashCodeExecutionToolResultBlockParam),
    TextEditorCodeExecutionToolResult(super::TextEditorCodeExecutionToolResultBlockParam),
    ToolSearchToolResult(super::ToolSearchToolResultBlockParam),
    ContainerUpload(super::ContainerUploadBlockParam),
}
