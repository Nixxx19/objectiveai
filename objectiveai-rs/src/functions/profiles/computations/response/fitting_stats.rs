use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, JsonSchema)]
#[schemars(rename = "FunctionsProfilesComputationsResponseFittingStats")]
pub struct FittingStats {
    pub loss: rust_decimal::Decimal,
    pub executions: usize,
    pub starts: usize,
    pub rounds: usize,
    pub errors: usize,
}
