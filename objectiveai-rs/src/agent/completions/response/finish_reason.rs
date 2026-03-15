//! Finish reason for agent completions.

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// The reason the model stopped generating.
#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq, JsonSchema, arbitrary::Arbitrary,
)]
#[schemars(rename = "agent.completions.response.FinishReason")]
pub enum FinishReason {
    /// The model reached a natural stop point or stop sequence.
    #[serde(rename = "stop")]
    Stop,
    /// The model reached the maximum token limit.
    #[serde(rename = "length")]
    Length,
    /// The model decided to call one or more tools.
    #[serde(rename = "tool_calls")]
    ToolCalls,
    /// The response was filtered due to content policy.
    #[serde(rename = "content_filter")]
    ContentFilter,
    /// An error occurred during generation.
    #[serde(rename = "error")]
    #[default]
    Error,
}
