mod completions;

pub use completions::*;

use serde::{Deserialize, Serialize};

/// Result of `agents get`.
///
/// Wire: `{"type":"notification","agent":{...GetAgentResponse...}}`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Agent {
    pub agent: objectiveai_sdk::agent::response::GetAgentResponse,
}
