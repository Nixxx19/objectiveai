use crate::chat::completions::response::util;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Usage {
    pub completion_tokens: u64,
    pub prompt_tokens: u64,
    pub total_tokens: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_tokens_details: Option<CompletionTokensDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_tokens_details: Option<PromptTokensDetails>,
    pub cost: rust_decimal::Decimal,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_details: Option<CostDetails>,
    pub total_cost: rust_decimal::Decimal,
    pub cost_multiplier: rust_decimal::Decimal,
    pub is_byok: bool,
}

impl Usage {
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
            (Some(self_cost_details), Some(other_cost_details)) => {
                self_cost_details.push(other_cost_details);
            }
            (None, Some(other_cost_details)) => {
                self.cost_details = Some(other_cost_details.clone());
            }
            _ => {}
        }
        self.total_cost += other.total_cost;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompletionTokensDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accepted_prediction_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rejected_prediction_tokens: Option<u64>,
}

impl CompletionTokensDetails {
    pub fn any_usage(&self) -> bool {
        self.accepted_prediction_tokens.is_some_and(|v| v > 0)
            || self.audio_tokens.is_some_and(|v| v > 0)
            || self.reasoning_tokens.is_some_and(|v| v > 0)
            || self.rejected_prediction_tokens.is_some_and(|v| v > 0)
    }

    pub fn push(&mut self, other: &CompletionTokensDetails) {
        util::push_option_u64(
            &mut self.accepted_prediction_tokens,
            &other.accepted_prediction_tokens,
        );
        util::push_option_u64(&mut self.audio_tokens, &other.audio_tokens);
        util::push_option_u64(
            &mut self.reasoning_tokens,
            &other.reasoning_tokens,
        );
        util::push_option_u64(
            &mut self.rejected_prediction_tokens,
            &other.rejected_prediction_tokens,
        );
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PromptTokensDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_write_tokens: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_tokens: Option<u64>,
}

impl PromptTokensDetails {
    pub fn any_usage(&self) -> bool {
        self.audio_tokens.is_some_and(|v| v > 0)
            || self.cached_tokens.is_some_and(|v| v > 0)
            || self.cache_write_tokens.is_some_and(|v| v > 0)
            || self.video_tokens.is_some_and(|v| v > 0)
    }

    pub fn push(&mut self, other: &PromptTokensDetails) {
        util::push_option_u64(&mut self.audio_tokens, &other.audio_tokens);
        util::push_option_u64(&mut self.cached_tokens, &other.cached_tokens);
        util::push_option_u64(
            &mut self.cache_write_tokens,
            &other.cache_write_tokens,
        );
        util::push_option_u64(&mut self.video_tokens, &other.video_tokens);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostDetails {
    pub upstream_inference_cost: rust_decimal::Decimal,
    pub upstream_upstream_inference_cost: rust_decimal::Decimal,
}

impl CostDetails {
    pub fn any_usage(&self) -> bool {
        self.upstream_inference_cost > rust_decimal::Decimal::ZERO
            || self.upstream_upstream_inference_cost
                > rust_decimal::Decimal::ZERO
    }

    pub fn push(&mut self, other: &CostDetails) {
        self.upstream_inference_cost += other.upstream_inference_cost;
        self.upstream_upstream_inference_cost +=
            other.upstream_upstream_inference_cost;
    }
}
