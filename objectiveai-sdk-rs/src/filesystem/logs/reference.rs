//! `LogReference` — the on-disk pointer shape every produced log
//! file uses to reference its children.
//!
//! On disk:
//!
//! ```json
//! { "type": "reference", "path": "agents/completions/messages/acc-1_0.json" }
//! ```
//!
//! Used by every chunk type's `produce_files` to represent the
//! parent → child relationship between fragmented log files. The
//! `path` is the relative on-disk path (under `${config_base_dir}/
//! logs/`) of the referenced child file.
//!
//! Optional fields (`index`, `task_index`, `task_path`,
//! `agent_index`, `swiss_pool_index`, `swiss_round`, `split_index`,
//! `error`, `output`) carry per-context metadata that some parents
//! attach to the reference. All optionals serialize-skip when `None`.
//!
//! Field declaration order matters: serde serializes in struct
//! order, so the keys appear on disk in the order below. That order
//! was chosen to match the legacy `serde_json::json!` / `Map::insert`
//! output of every produce_files implementation byte-for-byte.
//!
//! `error` and `output` are `serde_json::Value` because they hold
//! arbitrary task-output JSON whose schema isn't known at the SDK
//! layer.

use serde::{Deserialize, Serialize};

/// On-disk pointer from a parent log file to a child log file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogReference {
    #[serde(rename = "type")]
    pub r#type: LogReferenceTag,
    /// Relative on-disk path of the referenced file (under
    /// `${config_base_dir}/logs/`). Skipped when empty — the
    /// no-data sentinel case used by upstream chunks that wrap an
    /// optional inner (e.g. function-execution reasoning).
    #[serde(skip_serializing_if = "String::is_empty")]
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub index: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_index: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_path: Option<Vec<u64>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_index: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub swiss_pool_index: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub swiss_round: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub split_index: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub output: Option<serde_json::Value>,
}

impl LogReference {
    /// Build a plain reference with just `path` set. Optional
    /// metadata fields are populated by callers via direct field
    /// assignment.
    pub fn new(path: String) -> Self {
        Self {
            r#type: LogReferenceTag::Reference,
            path,
            index: None,
            task_index: None,
            task_path: None,
            agent_index: None,
            swiss_pool_index: None,
            swiss_round: None,
            split_index: None,
            error: None,
            output: None,
        }
    }
}

/// Constant `"reference"` discriminator — the `"type"` field on a
/// `LogReference`. Exists as its own type so the wire shape can't
/// drift to other strings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogReferenceTag {
    Reference,
}
