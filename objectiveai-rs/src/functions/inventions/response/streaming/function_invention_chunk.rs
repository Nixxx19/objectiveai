use crate::{agent, error, functions};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, arbitrary::Arbitrary)]
#[schemars(rename = "functions.inventions.response.streaming.FunctionInventionChunk")]
pub struct FunctionInventionChunk {
    pub id: String,
    pub completions: Vec<super::AgentCompletionChunk>,
    // yielded after steps with the current state
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<functions::inventions::State>,
    // yielded at the end
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<functions::RemoteFunctionPath>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<functions::FullRemoteFunction>,
    pub created: u64,
    pub object: super::Object,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<agent::completions::response::Usage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<error::ResponseError>,
}

impl FunctionInventionChunk {
    pub fn push(
        &mut self,
        FunctionInventionChunk {
            completions,
            state,
            path,
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
        if let Some(path) = path {
            self.path = Some(path.clone());
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
        other_completions: &[super::AgentCompletionChunk],
    ) {
        fn push_completion(
            completions: &mut Vec<super::AgentCompletionChunk>,
            other: &super::AgentCompletionChunk,
        ) {
            fn find_completion(
                completions: &mut Vec<super::AgentCompletionChunk>,
                index: u64,
            ) -> Option<&mut super::AgentCompletionChunk> {
                for completion in completions {
                    if completion.index == index {
                        return Some(completion);
                    }
                }
                None
            }
            if let Some(completion) = find_completion(completions, other.index)
            {
                completion.push(other);
            } else {
                completions.push(other.clone());
            }
        }
        for other_completion in other_completions {
            push_completion(&mut self.completions, other_completion);
        }
    }
}
