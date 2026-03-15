use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default, JsonSchema, arbitrary::Arbitrary)]
#[schemars(rename = "functions.profiles.computations.response.FittingStats")]
pub struct FittingStats {
    #[schemars(with = "f64")]
    pub loss: rust_decimal::Decimal,
    pub executions: usize,
    pub starts: usize,
    pub rounds: usize,
    pub errors: usize,
}
