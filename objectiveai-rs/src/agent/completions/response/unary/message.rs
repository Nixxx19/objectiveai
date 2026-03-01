//! Message type for unary agent completion responses.

use crate::agent::completions::response;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Message {
    Assistant(super::AssistantResponse),
    Tool(response::ToolResponse),
}

impl From<response::streaming::MessageChunk> for Message {
    fn from(chunk: response::streaming::MessageChunk) -> Self {
        match chunk {
            response::streaming::MessageChunk::Assistant(chunk) => {
                Message::Assistant(chunk.into())
            }
            response::streaming::MessageChunk::Tool(chunk) => {
                Message::Tool(chunk)
            }
        }
    }
}
