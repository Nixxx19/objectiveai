//! Effort settings for Agent output.

use serde::{Deserialize, Serialize};

/// The effort level for model output.
///
/// This setting hints to the model how detailed its responses should be.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Effort {
    /// Minimal output, concise responses.
    #[serde(rename = "low")]
    Low,
    /// Balanced output (default, normalized away during preparation).
    #[serde(rename = "medium")]
    Medium,
    /// Detailed output with thorough explanations.
    #[serde(rename = "high")]
    High,
    /// Maximum effort, most detailed output possible.
    #[serde(rename = "max")]
    Max,
}

impl Effort {
    /// Normalizes effort for deterministic hashing.
    ///
    /// The default `Medium` value is normalized to `None`.
    pub fn prepare(self) -> Option<Self> {
        if let Effort::Medium = self {
            None
        } else {
            Some(self)
        }
    }

    /// Validates the effort setting (always succeeds).
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }
}
