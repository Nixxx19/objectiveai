use serde::{Deserialize, Serialize};

/// Result of `swarms get`.
///
/// Wire: `{"type":"notification","swarm":{...GetSwarmResponse...}}`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Swarm {
    pub swarm: objectiveai_sdk::swarm::response::GetSwarmResponse,
}
