//! On-disk shape of a `LaboratoryExecutionChunk` log file.
//!
//! Mirrors [`super::LaboratoryExecutionChunk`] field-for-field, with
//! two type swaps: `builders: Vec<BuilderChunk>` → `Vec<LogReference>`
//! and `evaluations: Vec<EvaluationChunk>` → `Vec<LogReference>`.

use serde::Serialize;

use crate::agent;
use crate::error;
use crate::filesystem::logs::LogReference;

#[derive(Debug, Clone, Serialize)]
pub struct LaboratoryExecutionChunkLog {
    pub id: String,
    pub builders: Vec<LogReference>,
    pub evaluations: Vec<LogReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<error::ResponseError>,
    pub created: u64,
    pub object: super::Object,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<agent::completions::response::Usage>,
}
