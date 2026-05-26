//! On-disk shape of a `LaboratoryExecutionChunk` log file.
//!
//! Mirrors [`super::LaboratoryExecutionChunk`] field-for-field, with
//! two type swaps: `builders: Vec<BuilderChunk>` → `Vec<LogReference>`
//! and `evaluations: Vec<EvaluationChunk>` → `Vec<LogReference>`.

use schemars::JsonSchema;
use serde::Serialize;

use crate::agent;
use crate::error;
use crate::filesystem::logs::LogReference;

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[schemars(rename = "laboratories.executions.response.streaming.LaboratoryExecutionChunkLog")]
pub struct LaboratoryExecutionChunkLog {
    pub id: String,
    pub builders: Vec<LogReference>,
    pub evaluations: Vec<LogReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub error: Option<error::ResponseError>,
    pub created: u64,
    pub object: super::Object,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub usage: Option<agent::completions::response::Usage>,
}
