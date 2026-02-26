use crate::chat;
use crate::functions::inventions::response;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatCompletion {
    pub index: u64,
    #[serde(flatten)]
    pub inner: chat::completions::response::unary::ChatCompletion,
}

impl From<response::streaming::ChatCompletionChunk> for ChatCompletion {
    fn from(
        response::streaming::ChatCompletionChunk {
            index,
            inner,
        }: response::streaming::ChatCompletionChunk,
    ) -> Self {
        Self {
            index,
            inner: inner.into(),
        }
    }
}
