use crate::functions::inventions::recursive::response;
use crate::agent;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "functions.inventions.recursive.response.unary.FunctionInventionRecursive")]
pub struct FunctionInventionRecursive {
    pub id: String,
    pub inventions: Vec<super::FunctionInvention>,
    pub inventions_errors: bool,
    pub created: u64,
    pub object: super::Object,
    pub usage: agent::completions::response::Usage,
}

impl FunctionInventionRecursive {
    /// Normalize non-deterministic fields for test snapshot comparison.
    pub fn normalize_for_tests(&mut self) {
        self.id = String::new();
        self.created = 0;
        for invention in &mut self.inventions {
            invention.inner.normalize_for_tests();
        }
        // Sort inventions by state name and renumber indices sequentially.
        self.inventions.sort_by(|a, b| {
            a.inner.state.name().cmp(b.inner.state.name())
        });
        for (i, inv) in self.inventions.iter_mut().enumerate() {
            inv.index = i as u64;
        }
    }
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
