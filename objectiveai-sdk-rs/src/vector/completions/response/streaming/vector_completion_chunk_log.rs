//! On-disk shape of a `VectorCompletionChunk` log file.
//!
//! Mirrors [`super::VectorCompletionChunk`] field-for-field. The
//! one type swap is `completions: Vec<AgentCompletionChunk>` →
//! `Vec<LogReference>` (each per-agent completion is extracted to
//! its own file under `agents/completions/`). Field declaration
//! order matches the wire chunk so the legacy on-disk byte-shape
//! is preserved.

use serde::Serialize;

use crate::agent;
use crate::filesystem::logs::LogReference;
use crate::vector::completions::response;

#[derive(Debug, Clone, Serialize)]
pub struct VectorCompletionChunkLog {
    pub id: String,
    pub completions: Vec<LogReference>,
    pub votes: Vec<response::Vote>,
    pub scores: Vec<rust_decimal::Decimal>,
    pub weights: Vec<rust_decimal::Decimal>,
    pub created: u64,
    pub swarm: String,
    pub object: response::streaming::Object,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<agent::completions::response::Usage>,
}
