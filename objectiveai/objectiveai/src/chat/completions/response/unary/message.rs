use crate::chat::completions::response;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Message {
    pub content: Option<String>,
    pub refusal: Option<String>,
    pub role: response::Role,
    pub tool_calls: Option<Vec<super::ToolCall>>,

    // openrouter fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<response::Image>>,
}

impl From<response::streaming::Delta> for Message {
    fn from(
        response::streaming::Delta {
            content,
            refusal,
            role,
            tool_calls,
            reasoning,
            images,
        }: response::streaming::Delta,
    ) -> Self {
        Self {
            content,
            refusal,
            role: role.unwrap_or_default(),
            tool_calls: tool_calls.map(|tool_calls| {
                tool_calls.into_iter().map(super::ToolCall::from).collect()
            }),
            reasoning,
            images,
        }
    }
}
