use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Logprobs {
    pub content: Option<Vec<Logprob>>,
    pub refusal: Option<Vec<Logprob>>,
}

impl Logprobs {
    pub fn push(&mut self, other: &Logprobs) {
        match (&mut self.content, &other.content) {
            (Some(self_content), Some(other_content)) => {
                self_content.extend(other_content.clone());
            }
            (None, Some(other_content)) => {
                self.content = Some(other_content.clone());
            }
            _ => {}
        }
        match (&mut self.refusal, &other.refusal) {
            (Some(self_refusal), Some(other_refusal)) => {
                self_refusal.extend(other_refusal.clone());
            }
            (None, Some(other_refusal)) => {
                self.refusal = Some(other_refusal.clone());
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Logprob {
    pub token: String,
    pub bytes: Option<Vec<u8>>,
    pub logprob: rust_decimal::Decimal,
    pub top_logprobs: Vec<TopLogprob>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TopLogprob {
    pub token: String,
    pub bytes: Option<Vec<u8>>,
    pub logprob: Option<rust_decimal::Decimal>,
}
