//! Mock Agent types and validation logic.

use serde::{Deserialize, Serialize};
use twox_hash::XxHash3_128;

/// The base configuration for a Mock Agent (without computed ID).
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AgentBase {
    /// The upstream provider marker.
    pub upstream: super::Upstream,

    /// The output mode for vector completions. Ignored for agent completions.
    pub output_mode: super::OutputMode,

    /// If true, the mock client will return an error instead of a response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<bool>,
}

impl AgentBase {
    /// Normalizes the configuration for deterministic ID computation.
    pub fn prepare(&mut self) {
        if self.error == Some(false) {
            self.error = None;
        }
    }

    /// Validates the configuration.
    pub fn validate(&self) -> Result<(), String> {
        Ok(())
    }

    /// Computes the deterministic content-addressed ID.
    pub fn id(&self) -> String {
        let mut hasher = XxHash3_128::with_seed(0);
        hasher.write(serde_json::to_string(self).unwrap().as_bytes());
        format!("{:0>22}", base62::encode(hasher.finish_128()))
    }

    pub const fn model() -> &'static str {
        "mock"
    }
}

/// A validated Mock Agent with its computed content-addressed ID.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Agent {
    /// The deterministic content-addressed ID (22-character base62 string).
    pub id: String,
    /// The normalized configuration.
    #[serde(flatten)]
    pub base: AgentBase,
}

impl TryFrom<AgentBase> for Agent {
    type Error = String;
    fn try_from(mut base: AgentBase) -> Result<Self, Self::Error> {
        base.prepare();
        base.validate()?;
        let id = base.id();
        Ok(Agent { id, base })
    }
}
