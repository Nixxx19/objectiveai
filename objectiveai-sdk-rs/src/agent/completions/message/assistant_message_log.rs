//! `AssistantMessageLog` — on-disk shape of [`super::AssistantMessage`].
//! `content` becomes `Option<RichContentLog>` (extracted-to-files when
//! present); tool_calls, refusal, reasoning, name all stay inline.

use schemars::JsonSchema;
use serde::Serialize;

use super::{AssistantToolCall, RichContentLog};

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[schemars(rename = "agent.completions.message.AssistantMessageLog")]
pub struct AssistantMessageLog {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub content: Option<RichContentLog>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub refusal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub tool_calls: Option<Vec<AssistantToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub reasoning: Option<String>,
}
