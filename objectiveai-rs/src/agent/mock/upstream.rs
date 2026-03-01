//! Mock agent types.

use serde::{Deserialize, Serialize};

/// Mock upstream marker.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Upstream {
    #[default]
    Mock,
}
