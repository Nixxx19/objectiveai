use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub choices: Vec<objectiveai::chat::completions::response::streaming::Choice>,
    pub created: u64,
    pub model: String,
    pub object: objectiveai::chat::completions::response::streaming::Object,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<super::Usage>,

    // openrouter fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}

impl ChatCompletionChunk {
    pub fn into_downstream(
        self,
        id: String,
        model: String,
        is_byok: bool,
        cost_multiplier: rust_decimal::Decimal,
    ) -> objectiveai::chat::completions::response::streaming::ChatCompletionChunk {
        objectiveai::chat::completions::response::streaming::ChatCompletionChunk {
            id,
            upstream_id: self.id,
            choices: self.choices,
            created: self.created,
            model,
            upstream_model: self.model,
            object: self.object,
            service_tier: self.service_tier,
            system_fingerprint: self.system_fingerprint,
            usage: self
                .usage
                .map(|usage| usage.into_downstream(is_byok, cost_multiplier)),
            provider: self.provider,
        }
    }

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

    fn push_choices(
        &mut self,
        other_choices: &[objectiveai::chat::completions::response::streaming::Choice],
    ) {
        fn push_choice(
            choices: &mut Vec<objectiveai::chat::completions::response::streaming::Choice>,
            other: &objectiveai::chat::completions::response::streaming::Choice,
        ) {
            fn find_choice(
                choices: &mut Vec<objectiveai::chat::completions::response::streaming::Choice>,
                index: u64,
            ) -> Option<&mut objectiveai::chat::completions::response::streaming::Choice>
            {
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
