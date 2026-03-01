//! Upstream enumeration.

use serde::{Deserialize, Serialize};

/// Supported agent upstreams.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default,
)]
#[serde(rename_all = "snake_case")]
pub enum Upstream {
    /// Unknown Upstream.
    #[default]
    Unknown,
    /// OpenRouter Upstream.
    OpenRouter,
    /// Claude Agent SDK Upstream.
    ClaudeAgentSdk,
    /// Mock Upstream.
    Mock,
}

pub mod validate {
    use super::Upstream;

    pub fn validate(upstreams: &[Upstream]) -> Result<(), String> {
        // Check for duplicates
        let mut seen = std::collections::HashSet::new();
        for upstream in upstreams {
            if !seen.insert(upstream) {
                return Err(
                    "`upstreams` contains duplicate entries".to_string()
                );
            }
        }

        // Check for Unknown
        if seen.contains(&Upstream::Unknown) {
            return Err("`upstreams` contains unknown upstream".to_string());
        }

        Ok(())
    }
}
