use crate::chat::completions::response;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub upstream_id: String,
    pub choices: Vec<super::Choice>,
    pub created: u64,
    pub model: String,
    pub upstream_model: String,
    pub object: super::Object,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<response::Usage>,

    // openrouter fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}

impl ChatCompletionChunk {
    pub fn push(
        &mut self,
        ChatCompletionChunk {
            choices,
            service_tier,
            system_fingerprint,
            usage,
            provider,
            ..
        }: &ChatCompletionChunk,
    ) {
        self.push_choices(choices);
        if self.service_tier.is_none() {
            self.service_tier = service_tier.clone();
        }
        if self.system_fingerprint.is_none() {
            self.system_fingerprint = system_fingerprint.clone();
        }
        match (&mut self.usage, usage) {
            (Some(self_usage), Some(other_usage)) => {
                self_usage.push(other_usage);
            }
            (None, Some(other_usage)) => {
                self.usage = Some(other_usage.clone());
            }
            _ => {}
        }
        if self.provider.is_none() {
            self.provider = provider.clone();
        }
    }

    fn push_choices(&mut self, other_choices: &[super::Choice]) {
        fn push_choice(
            choices: &mut Vec<super::Choice>,
            other: &super::Choice,
        ) {
            fn find_choice(
                choices: &mut Vec<super::Choice>,
                index: u64,
            ) -> Option<&mut super::Choice> {
                for choice in choices {
                    if choice.index == index {
                        return Some(choice);
                    }
                }
                None
            }
            if let Some(choice) = find_choice(choices, other.index) {
                choice.push(other);
            } else {
                choices.push(other.clone());
            }
        }
        for other_choice in other_choices {
            push_choice(&mut self.choices, other_choice);
        }
    }
}
