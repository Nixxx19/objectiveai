use crate::functions;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "functions.inventions.state.AlphaScalarState")]
pub struct AlphaScalarState {
    #[serde(flatten)]
    pub params: super::Params,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema:
        Option<functions::alpha_scalar::expression::ScalarFunctionInputSchema>,
}
