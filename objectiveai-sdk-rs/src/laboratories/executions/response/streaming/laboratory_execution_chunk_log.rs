//! On-disk shape of a `LaboratoryExecutionChunk` log file.
//!
//! Mirrors [`super::LaboratoryExecutionChunk`] field-for-field with
//! two type swaps:
//! - `builders: Vec<BuilderChunk>` → `Vec<builder_log_reference::LogReference>`
//! - `evaluations: Vec<EvaluationChunk>` → `Vec<evaluation_log_reference::LogReference>`

use schemars::JsonSchema;
use serde::Serialize;

use crate::agent;
use crate::error;

use super::{builder_log_reference, evaluation_log_reference};

#[derive(Debug, Clone, Serialize, JsonSchema)]
#[schemars(rename = "laboratories.executions.response.streaming.LaboratoryExecutionChunkLog")]
pub struct LaboratoryExecutionChunkLog {
    pub id: String,
    pub builders: Vec<builder_log_reference::LogReference>,
    pub evaluations: Vec<evaluation_log_reference::LogReference>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub error: Option<error::ResponseError>,
    pub created: u64,
    pub object: super::Object,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub usage: Option<agent::completions::response::Usage>,
}
