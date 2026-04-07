use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum BetaContentBlock {
    Text(super::BetaTextBlock),
    Thinking(super::BetaThinkingBlock),
    RedactedThinking(super::BetaRedactedThinkingBlock),
    ToolUse(super::BetaToolUseBlock),
    ServerToolUse(super::BetaServerToolUseBlock),
    WebSearchToolResult(super::BetaWebSearchToolResultBlock),
    WebFetchToolResult(super::BetaWebFetchToolResultBlock),
    CodeExecutionToolResult(super::BetaCodeExecutionToolResultBlock),
    BashCodeExecutionToolResult(super::BetaBashCodeExecutionToolResultBlock),
    TextEditorCodeExecutionToolResult(super::BetaTextEditorCodeExecutionToolResultBlock),
    ToolSearchToolResult(super::BetaToolSearchToolResultBlock),
    MCPToolUse(super::BetaMCPToolUseBlock),
    MCPToolResult(super::BetaMCPToolResultBlock),
    ContainerUpload(super::BetaContainerUploadBlock),
    Compaction(super::BetaCompactionBlock),
}
