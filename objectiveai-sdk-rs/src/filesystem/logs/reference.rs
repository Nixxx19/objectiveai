//! `LogReference` — the plain on-disk pointer shape every produced
//! log file uses to reference a single child file.
//!
//! On disk:
//!
//! ```json
//! { "type": "reference", "path": "agents/completions/response/messages/acc-1_0.json" }
//! ```
//!
//! For references that carry additional per-context metadata (an
//! `index`, a `task_path`, an inline `error` or `output`, etc.),
//! each chunk that needs them defines its own `LogReference` struct
//! in a sibling `*_log_reference.rs` file — same name (`LogReference`),
//! different module path. See:
//!
//! - [`super::indexed_reference::LogReference`] — `{type, path, index}`
//! - `laboratories::executions::response::streaming::builder_log_reference::LogReference`
//! - `laboratories::executions::response::streaming::evaluation_log_reference::LogReference`
//! - `functions::executions::response::streaming::reasoning_summary_log_reference::LogReference`
//! - `functions::executions::response::streaming::function_execution_task_log_reference::LogReference`
//! - `functions::executions::response::streaming::vector_completion_task_log_reference::LogReference`
//! - `functions::executions::response::streaming::task_log_reference::LogReference` (untagged enum dispatch)

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Plain on-disk pointer (`type` + `path` only).
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "filesystem.logs.LogReference")]
pub struct LogReference {
    #[serde(rename = "type")]
    pub r#type: LogReferenceTag,
    /// Relative on-disk path of the referenced file (under
    /// `${config_base_dir}/logs/`). Skipped when empty — the
    /// no-data sentinel case used by some wrappers when the inner
    /// chunk has no content to log.
    #[serde(skip_serializing_if = "String::is_empty")]
    #[schemars(extend("omitempty" = true))]
    pub path: String,
}

impl LogReference {
    pub fn new(path: String) -> Self {
        Self {
            r#type: LogReferenceTag::Reference,
            path,
        }
    }
}

/// Constant `"reference"` discriminator — the `"type"` field on
/// every `LogReference` variant.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
#[schemars(rename = "filesystem.logs.LogReferenceTag")]
pub enum LogReferenceTag {
    Reference,
}
