use serde::{Deserialize, Serialize};

/// A typed item carried inside `item.started` / `item.updated` / `item.completed`
/// events. The wire shape is `{"type": <discriminator>, ...fields}` where the
/// discriminator is the `snake_case` form of the variant name (matching the
/// Python SDK's `_ITEM_MODELS` registry in `parsing.py`).
///
/// The [`Self::Unknown`] variant mirrors `UnknownThreadItem` in `types.py`:
/// any item whose `type` we don't recognise still parses, preserving the
/// raw payload for forward compatibility.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ThreadItem {
    Known(KnownThreadItem),
    Unknown(UnknownThreadItem),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum KnownThreadItem {
    AgentMessage(super::AgentMessageItem),
    Reasoning(super::ReasoningItem),
    CommandExecution(super::CommandExecutionItem),
    FileChange(super::FileChangeItem),
    McpToolCall(super::McpToolCallItem),
    WebSearch(super::WebSearchItem),
    TodoList(super::TodoListItem),
    Error(super::ErrorItem),
}

/// Forward-compat fallback for items with a `type` we don't recognise.
/// Mirrors `UnknownThreadItem` in `types.py:197-204`. Only `id` and the
/// discriminator are preserved — extra payload fields are discarded since
/// the consumer can't act on them anyway.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct UnknownThreadItem {
    pub id: String,
    pub r#type: String,
}
