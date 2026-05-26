//! On-disk shape of an `AgentCompletionChunk` log file.
//!
//! Mirrors [`super::AgentCompletionChunk`] field-for-field, with
//! two type swaps:
//!
//! - `messages: Vec<MessageChunk>` → `Vec<LogReference>` since each
//!   message is extracted to its own file.
//! - `continuation: Option<String>` → `Option<LogReference>` since
//!   the continuation token is extracted to its own file.
//!
//! Field declaration order matches the wire chunk so today's
//! `serde_json::to_value(&shell)` byte-shape is preserved.

use serde::Serialize;

use crate::agent;
use crate::agent::completions::response;
use crate::filesystem::logs::LogReference;

#[derive(Debug, Clone, Serialize)]
pub struct AgentCompletionChunkLog {
    pub id: String,
    pub created: u64,
    pub messages: Vec<LogReference>,
    pub object: response::streaming::Object,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<response::Usage>,
    pub upstream: agent::Upstream,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<crate::error::ResponseError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub continuation: Option<LogReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub messages_queued: Option<bool>,
}
