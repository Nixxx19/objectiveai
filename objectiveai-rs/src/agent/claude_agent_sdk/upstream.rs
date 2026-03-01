//! Claude Agent SDK agent types.

use serde::{Deserialize, Serialize};

/// Claude Agent SDK upstream marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Upstream {
    ClaudeAgentSdk,
}
