//! Mock Agent types and validation logic.

use serde::{Deserialize, Serialize};
use twox_hash::XxHash3_128;
use schemars::JsonSchema;

/// The base configuration for a Mock Agent (without computed ID).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, arbitrary::Arbitrary)]
#[schemars(rename = "agent.mock.AgentBase")]
pub struct AgentBase {
    /// The upstream provider marker.
    pub upstream: super::Upstream,

    /// The output mode for vector completions. Ignored for agent completions.
    pub output_mode: super::OutputMode,

    /// Number of top log probabilities to return (2-20).
    ///
    /// **Vector completions only.** Ignored for agent completions.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    #[arbitrary(with = crate::arbitrary_util::arbitrary_option_u64)]
    pub top_logprobs: Option<u64>,

    /// If true, the mock client will return an error instead of a response.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub error: Option<bool>,

    /// If true, this mock agent supports invention tool calling.
    /// Incompatible with output modes other than `instruction`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub invention: Option<bool>,
}

impl AgentBase {
    /// Normalizes the configuration for deterministic ID computation.
    pub fn prepare(&mut self) {
        self.top_logprobs = match self.top_logprobs {
            Some(0) | Some(1) => None,
            other => other,
        };
        if self.error == Some(false) {
            self.error = None;
        }
        if self.invention == Some(false) {
            self.invention = None;
        }
    }

    /// Validates the configuration.
    pub fn validate(&self) -> Result<(), String> {
        if let Some(top_logprobs) = self.top_logprobs
            && top_logprobs > 20
        {
            return Err("`top_logprobs` must be at most 20".to_string());
        }
        if self.invention == Some(true)
            && self.output_mode != super::OutputMode::Instruction
        {
            return Err(
                "`invention` is only compatible with `instruction` output mode"
                    .to_string(),
            );
        }
        Ok(())
    }

    /// Returns the messages as-is.
    pub fn merged_messages(
        &self,
        messages: Vec<super::super::completions::message::Message>,
    ) -> Vec<super::super::completions::message::Message> {
        messages
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "agent.mock.Agent")]
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
