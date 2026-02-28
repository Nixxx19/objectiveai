//! Object type for streaming agent completion responses.

use serde::{Deserialize, Serialize};

/// The object type for streaming agent completion chunks.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum Object {
    /// A agent completion chunk object.
    #[serde(rename = "agent.completion.chunk")]
    #[default]
    AgentCompletionChunk,
}
