use serde::{Deserialize, Serialize};

/// Single typed resource returned by a `*/get` endpoint.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "subkind", rename_all = "snake_case")]
pub enum Resource {
    /// Emitted by `agents get`.
    Agent(Box<objectiveai::agent::response::GetAgentResponse>),
    /// Emitted by `swarms get`.
    Swarm(Box<objectiveai::swarm::response::GetSwarmResponse>),
    /// Emitted by `functions get`.
    Function(Box<objectiveai::functions::response::GetFunctionResponse>),
    /// Emitted by `functions profiles get`.
    Profile(Box<objectiveai::functions::profiles::response::GetProfileResponse>),
    /// Emitted by `functions profiles pairs get`. The CLI fetches both
    /// halves and returns them together; we mirror that composite shape.
    Pair(Box<Pair>),
    /// Emitted by `functions inventions state get`.
    InventionState(
        Box<objectiveai::functions::inventions::state::response::GetFunctionInventionStateResponse>,
    ),
}

/// Function + profile composite returned by `functions profiles pairs get`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pair {
    pub function: objectiveai::functions::response::GetFunctionResponse,
    pub profile: objectiveai::functions::profiles::response::GetProfileResponse,
}
