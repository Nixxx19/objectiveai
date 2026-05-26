//! On-disk shape of an `AssistantResponseChunk` log file.
//!
//! Mirrors [`super::AssistantResponseChunk`] field-for-field, with
//! two type swaps:
//!
//! - `content: Option<RichContent>` → `Option<RichContentLog>` so
//!   media parts can be replaced by references.
//! - `logprobs: Option<Logprobs>` → `Option<LogReference>` since the
//!   logprobs payload is extracted to its own file.
//!
//! Field declaration order matches the wire chunk's order so the
//! on-disk JSON keys come out in the same sequence today's
//! `serde_json::to_value(&shell)` produces. Optional fields use
//! `skip_serializing_if = "Option::is_none"` everywhere the wire
//! chunk does, except for `finish_reason` which is always serialized
//! (even when `None` → JSON `null`) — same as the wire chunk.

use schemars::JsonSchema;
use serde::Serialize;

use crate::agent::completions::message;
use crate::agent::completions::response;
use crate::filesystem::logs::LogReference;

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[schemars(rename = "agent.completions.response.streaming.AssistantResponseChunkLog")]
pub struct AssistantResponseChunkLog {
    pub role: response::AssistantRole,
    pub index: u64,
    pub created: u64,
    pub agent: String,
    pub model: String,
    pub upstream_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub reasoning: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub tool_calls: Option<Vec<message::AssistantToolCallDelta>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub content: Option<message::RichContentLog>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub refusal: Option<String>,
    pub finish_reason: Option<response::FinishReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub logprobs: Option<LogReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub service_tier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub system_fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub usage: Option<response::UpstreamUsage>,
}
