use crate::vector::completions::response;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VectorCompletionChunk {
    pub id: String,
    pub completions: Vec<super::ChatCompletionChunk>,
    pub votes: Vec<response::Vote>,
    pub scores: Vec<rust_decimal::Decimal>,
    pub weights: Vec<rust_decimal::Decimal>,
    pub created: u64,
    pub ensemble: String,
    pub object: super::Object,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<response::Usage>,
}

impl VectorCompletionChunk {
    pub fn push(
        &mut self,
        VectorCompletionChunk {
            completions,
            votes,
            scores,
            weights,
            usage,
            ..
        }: &VectorCompletionChunk,
    ) {
        self.push_completions(completions);
        self.votes.extend_from_slice(votes);
        self.scores = scores.clone();
        self.weights = weights.clone();
        match (&mut self.usage, usage) {
            (Some(self_usage), Some(other_usage)) => {
                self_usage.push(other_usage);
            }
            (None, Some(other_usage)) => {
                self.usage = Some(other_usage.clone());
            }
            _ => {}
        }
    }

    fn push_completions(
        &mut self,
        other_completions: &[super::ChatCompletionChunk],
    ) {
        fn push_completion(
            completions: &mut Vec<super::ChatCompletionChunk>,
            other: &super::ChatCompletionChunk,
        ) {
            fn find_completion(
                completions: &mut Vec<super::ChatCompletionChunk>,
                index: u64,
            ) -> Option<&mut super::ChatCompletionChunk> {
                for completion in completions {
                    if completion.index == index {
                        return Some(completion);
                    }
                }
                None
            }
            if let Some(completion) = find_completion(completions, other.index)
            {
                completion.push(other);
            } else {
                completions.push(other.clone());
            }
        }
        for other_completion in other_completions {
            push_completion(&mut self.completions, other_completion);
        }
    }
}
