use crate::chat;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Usage {
    pub completion_tokens: u64,
    pub prompt_tokens: u64,
    pub total_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_tokens_details:
        Option<chat::completions::response::CompletionTokensDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_tokens_details:
        Option<chat::completions::response::PromptTokensDetails>,
    pub cost: rust_decimal::Decimal,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_details: Option<chat::completions::response::CostDetails>,
    pub total_cost: rust_decimal::Decimal,
}

impl Usage {
    pub fn any_usage(&self) -> bool {
        self.completion_tokens > 0
            || self.prompt_tokens > 0
            || self.total_tokens > 0
            || self.completion_tokens_details.as_ref().is_some_and(
                chat::completions::response::CompletionTokensDetails::any_usage,
            )
            || self.prompt_tokens_details.as_ref().is_some_and(
                chat::completions::response::PromptTokensDetails::any_usage,
            )
            || self.cost > rust_decimal::Decimal::ZERO
            || self.cost_details.as_ref().is_some_and(
                chat::completions::response::CostDetails::any_usage,
            )
            || self.total_cost > rust_decimal::Decimal::ZERO
    }

    pub fn push(&mut self, other: &Usage) {
        self.completion_tokens += other.completion_tokens;
        self.prompt_tokens += other.prompt_tokens;
        self.total_tokens += other.total_tokens;
        match (
            &mut self.completion_tokens_details,
            &other.completion_tokens_details,
        ) {
            (Some(self_details), Some(other_details)) => {
                self_details.push(other_details);
            }
            (None, Some(other_details)) => {
                self.completion_tokens_details = Some(other_details.clone());
            }
            _ => {}
        }
        match (
            &mut self.prompt_tokens_details,
            &other.prompt_tokens_details,
        ) {
            (Some(self_details), Some(other_details)) => {
                self_details.push(other_details);
            }
            (None, Some(other_details)) => {
                self.prompt_tokens_details = Some(other_details.clone());
            }
            _ => {}
        }
        self.cost += other.cost;
        match (&mut self.cost_details, &other.cost_details) {
            (Some(self_details), Some(other_details)) => {
                self_details.push(other_details);
            }
            (None, Some(other_details)) => {
                self.cost_details = Some(other_details.clone());
            }
            _ => {}
        }
        self.total_cost += other.total_cost;
    }

    pub fn push_chat_completion_usage(
        &mut self,
        other: &chat::completions::response::Usage,
    ) {
        self.completion_tokens += other.completion_tokens;
        self.prompt_tokens += other.prompt_tokens;
        self.total_tokens += other.total_tokens;
        match (
            &mut self.completion_tokens_details,
            &other.completion_tokens_details,
        ) {
            (Some(self_details), Some(other_details)) => {
                self_details.push(other_details);
            }
            (None, Some(other_details)) => {
                self.completion_tokens_details = Some(other_details.clone());
            }
            _ => {}
        }
        match (
            &mut self.prompt_tokens_details,
            &other.prompt_tokens_details,
        ) {
            (Some(self_details), Some(other_details)) => {
                self_details.push(other_details);
            }
            (None, Some(other_details)) => {
                self.prompt_tokens_details = Some(other_details.clone());
            }
            _ => {}
        }
        self.cost += other.cost;
        match (&mut self.cost_details, &other.cost_details) {
            (Some(self_details), Some(other_details)) => {
                self_details.push(other_details);
            }
            (None, Some(other_details)) => {
                self.cost_details = Some(other_details.clone());
            }
            _ => {}
        }
        self.total_cost += other.total_cost;
    }
}
