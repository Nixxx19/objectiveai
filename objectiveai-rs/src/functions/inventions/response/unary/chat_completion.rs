use crate::chat;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatCompletion {
    pub index: u64,
    #[serde(flatten)]
    pub inner: chat::completions::response::unary::ChatCompletion,
}
