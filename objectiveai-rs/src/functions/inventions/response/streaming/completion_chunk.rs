use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CompletionChunk {
    Chat(super::ChatCompletionChunk),
    Tool(super::super::ToolResponse),
}

impl CompletionChunk {
    pub fn index(&self) -> u64 {
        match self {
            CompletionChunk::Chat(chat) => chat.index,
            CompletionChunk::Tool(tool) => tool.index,
        }
    }

    pub fn push(&mut self, other: &CompletionChunk) {
        match (self, other) {
            (CompletionChunk::Chat(self_chat), CompletionChunk::Chat(other_chat)) => {
                self_chat.push(other_chat);
            }
            _ => {}
        }
    }
}
