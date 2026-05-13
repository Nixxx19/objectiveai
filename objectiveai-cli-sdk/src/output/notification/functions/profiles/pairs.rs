use serde::{Deserialize, Serialize};

/// Result of `functions profiles pairs get`. The CLI fetches both
/// halves of the pair and emits them together.
///
/// Wire: `{"type":"notification","pair":{"function":...,"profile":...}}`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pair {
    pub pair: FunctionProfilePair,
}

/// The composite body inside a `Pair` notification.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FunctionProfilePair {
    pub function: objectiveai_sdk::functions::response::GetFunctionResponse,
    pub profile: objectiveai_sdk::functions::profiles::response::GetProfileResponse,
}
