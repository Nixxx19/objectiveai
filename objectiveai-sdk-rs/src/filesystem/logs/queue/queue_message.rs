//! `QueueMessage` — per-role message shape inside a
//! [`super::QueueItem::UserRequest`]'s `messages` list. Mirrors the
//! per-role `*MessageLog` types with content/refusal/reasoning/tool_calls
//! flattened to bare [`Id`] paths.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{Content, Id};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(rename = "filesystem.logs.queue.QueueMessage")]
pub enum QueueMessage {
    #[schemars(title = "DeveloperMessage")]
    DeveloperMessage {
        content: Content,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[schemars(extend("omitempty" = true))]
        name: Option<String>,
    },
    #[schemars(title = "SystemMessage")]
    SystemMessage {
        content: Content,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[schemars(extend("omitempty" = true))]
        name: Option<String>,
    },
    #[schemars(title = "UserMessage")]
    UserMessage {
        content: Content,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[schemars(extend("omitempty" = true))]
        name: Option<String>,
    },
    #[schemars(title = "AssistantMessage")]
    AssistantMessage {
        #[serde(skip_serializing_if = "Option::is_none")]
        #[schemars(extend("omitempty" = true))]
        content: Option<Content>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[schemars(extend("omitempty" = true))]
        name: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[schemars(extend("omitempty" = true))]
        reasoning: Option<Id>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[schemars(extend("omitempty" = true))]
        tool_calls: Option<Vec<Id>>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[schemars(extend("omitempty" = true))]
        refusal: Option<Id>,
    },
    #[schemars(title = "ToolMessage")]
    ToolMessage {
        content: Content,
        tool_call_id: String,
    },
}
