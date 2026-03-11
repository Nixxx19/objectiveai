use crate::functions;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "FunctionsInventionsStateAlphaVectorState")]
pub struct AlphaVectorState {
    #[serde(flatten)]
    pub params: super::Params,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_schema:
        Option<functions::alpha_vector::expression::VectorFunctionInputSchema>,
}
