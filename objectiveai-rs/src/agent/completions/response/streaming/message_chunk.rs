use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(rename = "AgentCompletionsResponseStreamingMessageChunk")]
pub enum MessageChunk {
    Assistant(super::AssistantResponseChunk),
    Tool(super::super::ToolResponse),
}

impl MessageChunk {
    pub fn index(&self) -> u64 {
        match self {
            MessageChunk::Assistant(chunk) => chunk.index,
            MessageChunk::Tool(chunk) => chunk.index,
        }
    }

    pub fn push(&mut self, other: &MessageChunk) {
        match (self, other) {
            (
                MessageChunk::Assistant(self_chunk),
                MessageChunk::Assistant(other_chunk),
            ) => {
                self_chunk.push(other_chunk);
            }
            _ => panic!("cannot push to or from tool chunks"),
        }
    }
}
