use crate::chat;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatCompletionChunk {
    pub index: u64,
    #[serde(flatten)]
    pub inner: chat::completions::response::streaming::ChatCompletionChunk,
}

impl ChatCompletionChunk {
    pub fn push(&mut self, other: &ChatCompletionChunk) {
        self.inner.push(&other.inner);
    }
}
