use crate::chat::completions::response;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Choice {
    pub message: super::Message,
    pub finish_reason: response::FinishReason,
    pub index: u64,
    pub logprobs: Option<response::Logprobs>,
}

impl From<response::streaming::Choice> for Choice {
    fn from(
        response::streaming::Choice {
            delta,
            finish_reason,
            index,
            logprobs,
        }: response::streaming::Choice,
    ) -> Self {
        Self {
            message: super::Message::from(delta),
            finish_reason: finish_reason.unwrap_or_default(),
            index,
            logprobs,
        }
    }
}
