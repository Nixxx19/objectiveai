//! On-disk shape of a `ToolResponse` log file.
//!
//! Mirrors [`super::ToolResponse`]'s flattened shape (`role`, `index`,
//! then `ToolMessage`'s fields hoisted via `serde(flatten)`). One
//! type swap: `content: RichContent` → `RichContentLog` so media
//! parts can be replaced by references.

use serde::Serialize;

use crate::agent::completions::message::RichContentLog;

use super::ToolRole;

#[derive(Debug, Clone, Serialize)]
pub struct ToolResponseLog {
    pub role: ToolRole,
    pub index: u64,
    pub content: RichContentLog,
    pub tool_call_id: String,
}
