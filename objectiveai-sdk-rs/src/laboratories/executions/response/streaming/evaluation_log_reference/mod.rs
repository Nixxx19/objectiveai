//! `LogReference` for evaluation-completion entries within a
//! laboratory execution log file. Adds an optional `output` carrying
//! the evaluator's scoreable result.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::filesystem::logs::LogReferenceTag;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "laboratories.executions.response.streaming.evaluation_log_reference.LogReference")]
pub struct LogReference {
    #[serde(rename = "type")]
    pub r#type: LogReferenceTag,
    #[serde(skip_serializing_if = "String::is_empty")]
    #[schemars(extend("omitempty" = true))]
    pub path: String,
    pub index: u64,
    pub agent_index: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub output: Option<serde_json::Value>,
}

impl LogReference {
    pub fn new(path: String, index: u64, agent_index: u64) -> Self {
        Self {
            r#type: LogReferenceTag::Reference,
            path,
            index,
            agent_index,
            output: None,
        }
    }
}
