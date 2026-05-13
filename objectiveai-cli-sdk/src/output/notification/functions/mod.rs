mod executions;
mod inventions;
mod profiles;

pub use executions::*;
pub use inventions::*;
pub use profiles::*;

use serde::{Deserialize, Serialize};

/// Result of `functions get`.
///
/// Wire: `{"type":"notification","function":{...GetFunctionResponse...}}`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Function {
    pub function: objectiveai::functions::response::GetFunctionResponse,
}
