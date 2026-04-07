use crate::functions;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, arbitrary::Arbitrary)]
#[schemars(rename = "functions.inventions.recursive.response.streaming.FunctionInventionChunk")]
pub struct FunctionInventionChunk {
    #[arbitrary(with = crate::arbitrary_util::arbitrary_u64)]
    pub index: u64,
    #[serde(flatten)]
    pub inner:
        functions::inventions::response::streaming::FunctionInventionChunk,
}

impl FunctionInventionChunk {
    pub fn push(&mut self, other: &FunctionInventionChunk) {
        self.inner.push(&other.inner);
    }
}
