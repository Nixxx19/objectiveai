//! Reasoning configuration for function executions.

use crate::agent;
use serde::{Deserialize, Serialize};

/// Configuration for generating reasoning summaries during execution.
///
/// When enabled, an LLM summarizes the execution's reasoning process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reasoning {
    /// The primary agent to use for generating reasoning summaries.
    pub agent: agent::Agent,
    /// Fallback agents tried in order if the primary is rate-limited or errors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agents: Option<Vec<agent::Agent>>,
}
