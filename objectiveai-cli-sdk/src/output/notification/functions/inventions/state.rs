use serde::{Deserialize, Serialize};

/// Result of `functions inventions state get`.
///
/// Wire: `{"type":"notification","state":{...GetFunctionInventionStateResponse...}}`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct State {
    pub state: objectiveai_sdk::functions::inventions::state::response::GetFunctionInventionStateResponse,
}
