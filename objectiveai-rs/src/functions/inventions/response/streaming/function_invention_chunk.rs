use crate::{error, functions, vector};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInventionChunk {
    pub id: String,
    pub completions: Vec<super::CompletionChunk>,
    // yielded after steps with the current state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<functions::inventions::State>,
    // yielded at the end
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<functions::AlphaRemoteFunction>,
    pub created: u64,
    pub object: super::Object,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<vector::completions::response::Usage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<error::ResponseError>,
}

impl FunctionInventionChunk {
    pub fn push(
        &mut self,
        FunctionInventionChunk {
            completions,
            state,
            function,
            usage,
            error,
            ..
        }: &FunctionInventionChunk,
    ) {
        self.push_completions(completions);
        if let Some(state) = state {
            self.state = Some(state.clone());
        }
        if let Some(function) = function {
            self.function = Some(function.clone());
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
        if let Some(error) = error {
            self.error = Some(error.clone());
        }
    }

    fn push_completions(
        &mut self,
        other_completions: &[super::CompletionChunk],
    ) {
        fn push_completion(
            completions: &mut Vec<super::CompletionChunk>,
            other: &super::CompletionChunk,
        ) {
            fn find_completion(
                completions: &mut Vec<super::CompletionChunk>,
                index: u64,
            ) -> Option<&mut super::CompletionChunk> {
                for completion in completions {
                    if completion.index() == index {
                        return Some(completion);
                    }
                }
                None
            }
            if let Some(existing) =
                find_completion(completions, other.index())
            {
                existing.push(other);
            } else {
                completions.push(other.clone());
            }
        }
        for other in other_completions {
            push_completion(&mut self.completions, other);
        }
    }
}
