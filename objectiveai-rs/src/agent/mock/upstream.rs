//! Mock agent types.

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Mock upstream marker.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(rename = "AgentMockUpstream")]
pub enum Upstream {
    #[default]
    Mock,
}
