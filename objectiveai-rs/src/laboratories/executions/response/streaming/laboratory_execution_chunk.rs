use crate::{agent, error};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Streaming chunk for a laboratory execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, arbitrary::Arbitrary)]
#[schemars(rename = "laboratories.executions.response.streaming.LaboratoryExecutionChunk")]
pub struct LaboratoryExecutionChunk {
    pub id: String,
    pub builders: Vec<super::BuilderChunk>,
    pub evaluations: Vec<super::EvaluationChunk>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub error: Option<error::ResponseError>,
    #[arbitrary(with = crate::arbitrary_util::arbitrary_u64)]
    pub created: u64,
    pub object: super::Object,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub usage: Option<agent::completions::response::Usage>,
}

impl LaboratoryExecutionChunk {
    pub fn push(
        &mut self,
        LaboratoryExecutionChunk {
            builders,
            evaluations,
            error,
            usage,
            ..
        }: &LaboratoryExecutionChunk,
    ) {
        self.push_builders(builders);
        self.push_evaluations(evaluations);
        if let Some(error) = error {
            self.error = Some(error.clone());
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
    }

    fn push_builders(&mut self, others: &[super::BuilderChunk]) {
        for other in others {
            if let Some(existing) = self.builders.iter_mut().find(|c| c.index == other.index) {
                existing.push(other);
            } else {
                self.builders.push(other.clone());
            }
        }
    }

    fn push_evaluations(&mut self, others: &[super::EvaluationChunk]) {
        for other in others {
            if let Some(existing) = self
                .evaluations
                .iter_mut()
                .find(|c| c.index == other.index)
            {
                existing.push(other);
            } else {
                self.evaluations.push(other.clone());
            }
        }
    }
}
