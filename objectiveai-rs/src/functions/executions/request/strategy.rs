use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(rename = "FunctionsExecutionsRequestStrategy")]
pub enum Strategy {
    /// Scalar or Vector
    Default,
    /// Vector
    SwissSystem {
        /// How many vector responses for each execution
        pool: Option<usize>, // default is 10
        /// How many sequential rounds of comparison
        rounds: Option<usize>, // default is 3
    },
}
