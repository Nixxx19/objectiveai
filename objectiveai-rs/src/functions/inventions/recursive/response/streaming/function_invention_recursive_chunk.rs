use crate::agent;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "FunctionsInventionsRecursiveResponseStreamingFunctionInventionRecursiveChunk")]
pub struct FunctionInventionRecursiveChunk {
    pub id: String,
    pub inventions: Vec<super::FunctionInventionChunk>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inventions_errors: Option<bool>,
    pub created: u64,
    pub object: super::Object,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<agent::completions::response::Usage>,
}

impl FunctionInventionRecursiveChunk {
    pub fn push(
        &mut self,
        FunctionInventionRecursiveChunk {
            inventions,
            inventions_errors,
            usage,
            ..
        }: &FunctionInventionRecursiveChunk,
    ) {
        self.push_inventions(inventions);
        if let Some(true) = inventions_errors {
            self.inventions_errors = Some(true);
        }
        match (&mut self.usage, usage) {
            (Some(self_usage), Some(other_usage)) => {
                self_usage.push(other_usage);
            }
            (None, Some(other_usage)) => {
                self.usage = Some(other_usage.clone());
            }
            _ => {}
        }
    }

    fn push_inventions(
        &mut self,
        other_inventions: &[super::FunctionInventionChunk],
    ) {
        fn push_invention(
            inventions: &mut Vec<super::FunctionInventionChunk>,
            other: &super::FunctionInventionChunk,
        ) {
            fn find_invention(
                inventions: &mut Vec<super::FunctionInventionChunk>,
                index: u64,
            ) -> Option<&mut super::FunctionInventionChunk> {
                for invention in inventions {
                    if invention.index == index {
                        return Some(invention);
                    }
                }
                None
            }
            if let Some(existing) = find_invention(inventions, other.index) {
                existing.push(other);
            } else {
                inventions.push(other.clone());
            }
        }
        for other in other_inventions {
            push_invention(&mut self.inventions, other);
        }
    }
}
