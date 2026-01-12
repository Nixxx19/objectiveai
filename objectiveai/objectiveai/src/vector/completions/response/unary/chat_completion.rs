use crate::{chat, error, vector::completions::response};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatCompletion {
    pub index: u64,
    #[serde(flatten)]
    pub inner: chat::completions::response::unary::ChatCompletion,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<error::ResponseError>,
}

impl From<response::streaming::ChatCompletionChunk> for ChatCompletion {
    fn from(
        response::streaming::ChatCompletionChunk {
            index,
            inner,
            error,
        }: response::streaming::ChatCompletionChunk,
    ) -> Self {
        Self {
            index,
            inner: chat::completions::response::unary::ChatCompletion::from(
                inner,
            ),
            error,
        }
    }
}
