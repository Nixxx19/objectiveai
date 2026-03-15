use crate::functions;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

pub type Dataset = Vec<DatasetItem>;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "functions.profiles.computations.request.DatasetItem")]
pub struct DatasetItem {
    pub input: functions::expression::InputValue,
    pub target: Target,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
#[schemars(rename = "functions.profiles.computations.request.Target")]
pub enum Target {
    Scalar { #[schemars(with = "f64")] value: rust_decimal::Decimal }, // desired scalar output
    Vector { #[schemars(with = "Vec<f64>")] value: Vec<rust_decimal::Decimal> }, // desired vector output
    VectorWinner { value: usize }, // desired winning index in vector completion
}
