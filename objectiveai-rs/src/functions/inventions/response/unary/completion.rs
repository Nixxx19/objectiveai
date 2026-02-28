use crate::functions::inventions::response;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Completion {
    Agent(super::AgentCompletion),
    Tool(super::super::ToolResponse),
}

impl From<response::streaming::CompletionChunk> for Completion {
    fn from(chunk: response::streaming::CompletionChunk) -> Self {
        match chunk {
            response::streaming::CompletionChunk::Agent(chunk) => {
                Completion::Agent(chunk.into())
            }
            response::streaming::CompletionChunk::Tool(tool) => {
                Completion::Tool(tool)
            }
        }
    }
}
