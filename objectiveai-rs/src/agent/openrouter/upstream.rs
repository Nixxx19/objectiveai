//! OpenRouter agent types.

use serde::{Deserialize, Serialize};

/// OpenRouter upstream marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Upstream {
    Openrouter,
}
