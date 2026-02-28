use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CompletionChunk {
    Agent(super::AgentCompletionChunk),
    Tool(super::super::ToolResponse),
}

impl CompletionChunk {
    pub fn index(&self) -> u64 {
        match self {
            CompletionChunk::Agent(agent) => agent.index,
            CompletionChunk::Tool(tool) => tool.index,
        }
    }

    pub fn push(&mut self, other: &CompletionChunk) {
        match (self, other) {
            (CompletionChunk::Agent(self_agent), CompletionChunk::Agent(other_agent)) => {
                self_agent.push(other_agent);
            }
            _ => {}
        }
    }
}
