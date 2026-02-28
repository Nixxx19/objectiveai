//! Model specification for agent completion requests.

use serde::{Deserialize, Serialize};

/// The model to use for agent completion.
///
/// Can be either:
/// - An inline [`AgentBase`](super::super::super::AgentBase) configuration
/// - The ID of a previously used Agent (22-character base62 string)
///
/// Since IDs are content-addressed, ObjectiveAI stores Agent definitions
/// when they are successfully used. "Previously used" means the ID exists in
/// ObjectiveAI's database from any successful use by anyone.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Model {
    /// The content-addressed ID of an Agent stored in ObjectiveAI's database.
    Id(String),
    /// An inline Agent configuration.
    Provided(super::super::super::AgentBase),
}
