use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CompletionChunk {
    Chat(super::ChatCompletionChunk),
    Tool(super::super::ToolResponse),
}
