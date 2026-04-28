use serde::{Deserialize, Serialize};

/// A typed item carried inside `item.started` / `item.updated` / `item.completed`
/// events. The wire shape is `{"type": <discriminator>, ...fields}` where the
/// discriminator is the `snake_case` form of the variant name (matching the
/// Python SDK's `_ITEM_MODELS` registry in `parsing.py`).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ThreadItem {
    AgentMessage(super::AgentMessageItem),
    Reasoning(super::ReasoningItem),
    CommandExecution(super::CommandExecutionItem),
    FileChange(super::FileChangeItem),
    McpToolCall(super::McpToolCallItem),
    WebSearch(super::WebSearchItem),
    TodoList(super::TodoListItem),
    Error(super::ErrorItem),
}
