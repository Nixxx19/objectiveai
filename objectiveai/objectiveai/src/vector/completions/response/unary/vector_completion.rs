use crate::vector::completions::response;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VectorCompletion {
    pub id: String,
    pub completions: Vec<super::ChatCompletion>,
    pub votes: Vec<response::Vote>,
    pub scores: Vec<rust_decimal::Decimal>,
    pub weights: Vec<rust_decimal::Decimal>,
    pub created: u64,
    pub ensemble: String,
    pub object: super::Object,
    pub usage: response::Usage,
}

impl From<response::streaming::VectorCompletionChunk> for VectorCompletion {
    fn from(
        response::streaming::VectorCompletionChunk {
            id,
            completions,
            votes,
            scores,
            weights,
            created,
            ensemble,
            object,
            usage,
        }: response::streaming::VectorCompletionChunk,
    ) -> Self {
        Self {
            id,
            completions: completions
                .into_iter()
                .map(super::ChatCompletion::from)
                .collect(),
            votes,
            scores,
            weights,
            created,
            ensemble,
            object: object.into(),
            usage: usage.unwrap_or_default(),
        }
    }
}
