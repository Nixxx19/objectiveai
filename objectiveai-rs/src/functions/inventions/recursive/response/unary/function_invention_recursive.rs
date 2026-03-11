use crate::functions::inventions::recursive::response;
use crate::agent;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "FunctionsInventionsRecursiveResponseUnaryFunctionInventionRecursive")]
pub struct FunctionInventionRecursive {
    pub id: String,
    pub inventions: Vec<super::FunctionInvention>,
    pub inventions_errors: bool,
    pub created: u64,
    pub object: super::Object,
    pub usage: agent::completions::response::Usage,
}

impl From<response::streaming::FunctionInventionRecursiveChunk>
    for FunctionInventionRecursive
{
    fn from(
        response::streaming::FunctionInventionRecursiveChunk {
            id,
            inventions,
            inventions_errors,
            created,
            object,
            usage,
        }: response::streaming::FunctionInventionRecursiveChunk,
    ) -> Self {
        Self {
            id,
            inventions: inventions
                .into_iter()
                .map(super::FunctionInvention::from)
                .collect(),
            inventions_errors: inventions_errors.unwrap_or(false),
            created,
            object: object.into(),
            usage: usage.unwrap_or_default(),
        }
    }
}
