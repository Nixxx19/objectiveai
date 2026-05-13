mod pairs;

pub use pairs::*;

use serde::{Deserialize, Serialize};

/// Result of `functions profiles get`.
///
/// Wire: `{"type":"notification","profile":{...GetProfileResponse...}}`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Profile {
    pub profile: objectiveai::functions::profiles::response::GetProfileResponse,
}
