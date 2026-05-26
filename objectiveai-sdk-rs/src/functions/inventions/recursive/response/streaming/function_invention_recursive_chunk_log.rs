//! On-disk shape of a `FunctionInventionRecursiveChunk` log file.
//!
//! Mirrors [`super::FunctionInventionRecursiveChunk`] field-for-field,
//! with `inventions: Vec<FunctionInventionChunk>` → `Vec<LogReference>`.

use serde::Serialize;

use crate::agent;
use crate::filesystem::logs::LogReference;

#[derive(Debug, Clone, Serialize)]
pub struct FunctionInventionRecursiveChunkLog {
    pub id: String,
    pub inventions: Vec<LogReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inventions_errors: Option<bool>,
    pub created: u64,
    pub object: super::Object,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<agent::completions::response::Usage>,
}
