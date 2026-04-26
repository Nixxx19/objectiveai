use serde::{Deserialize, Serialize};

use super::id_prefix::IdPrefix;
use super::scanner::scan_id_prefix;
use super::{StdioDiagLevel, StdioParseError};

/// One line emitted by the runner on stderr.
///
/// Two variants:
///
/// - [`StdioError::Diag`] is per-request and carries `id`. It's the
///   normal channel for non-fatal warnings (e.g. rate-limit retry
///   notes). The `id` is always the second field on the wire.
/// - [`StdioError::Fatal`] is process-level and carries no `id`. The
///   runner only emits one of these on its way to a non-zero exit
///   (import failure, asyncio init crash, etc.). Once `main_loop` is
///   running, no untagged stderr line is ever emitted.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StdioError {
    /// Per-request diagnostic, tagged with `id`.
    Diag {
        id: String,
        level: StdioDiagLevel,
        message: String,
    },
    /// Process-level fatal — runner is exiting non-zero. Untagged.
    Fatal { message: String },
}

impl StdioError {
    /// Parse `line` IFF it is either:
    ///
    /// - a per-request `diag` whose `id` (second field, after `type`)
    ///   equals `request_id`, or
    /// - an untagged `fatal` (which is process-level and carries no
    ///   `id` — always relevant regardless of `request_id`).
    ///
    /// Returns `Ok(None)` for a `diag` line tagged with someone
    /// else's id. Returns `Err(_)` for a malformed line.
    pub fn try_parse(
        line: &str,
        request_id: &str,
    ) -> Result<Option<Self>, StdioParseError> {
        match scan_id_prefix(line)? {
            IdPrefix::Id(id) if id == request_id => {
                let parsed = serde_json::from_str(line)?;
                Ok(Some(parsed))
            }
            IdPrefix::Id(_) => Ok(None),
            IdPrefix::NoId => {
                // Second field is not `id` — must be the untagged
                // `fatal` carve-out (process-level).
                let parsed = serde_json::from_str(line)?;
                Ok(Some(parsed))
            }
        }
    }
}
