//! Claude Agent SDK agent types.

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Claude Agent SDK upstream marker.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(rename = "AgentClaudeAgentSdkUpstream")]
pub enum Upstream {
    #[default]
    ClaudeAgentSdk,
}
