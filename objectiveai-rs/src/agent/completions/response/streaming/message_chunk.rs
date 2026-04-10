use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, arbitrary::Arbitrary)]
#[serde(untagged)]
#[schemars(rename = "agent.completions.response.streaming.MessageChunk")]
pub enum MessageChunk {
    #[schemars(title = "Assistant")]
    Assistant(super::AssistantResponseChunk),
    #[schemars(title = "Tool")]
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
            // _ => panic!("cannot push to or from tool chunks"),
            _ => {}
        }
    }

    /// Produces log files for this message.
    ///
    /// Returns `(reference, files)` where `reference` is a
    /// `{"type": "reference", "path": ...}` JSON value, and `files`
    /// contains all produced files.
    #[cfg(feature = "filesystem")]
    pub fn produce_files(&self, id: &str, prefix: &str) -> (serde_json::Value, Vec<(String, Vec<u8>)>) {
        match self {
            MessageChunk::Assistant(chunk) => chunk.produce_files(id, prefix),
            MessageChunk::Tool(chunk) => chunk.produce_files(id, prefix),
        }
    }
}
