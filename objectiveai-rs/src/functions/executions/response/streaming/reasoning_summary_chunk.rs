use crate::{agent, error};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, JsonSchema, arbitrary::Arbitrary)]
#[schemars(rename = "functions.executions.response.streaming.ReasoningSummaryChunk")]
pub struct ReasoningSummaryChunk {
    #[serde(flatten)]
    pub inner: agent::completions::response::streaming::AgentCompletionChunk,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<error::ResponseError>,
}

impl ReasoningSummaryChunk {
    pub fn push(&mut self, other: &ReasoningSummaryChunk) {
        self.inner.push(&other.inner);
        if let Some(error) = &other.error {
            self.error = Some(error.clone());
        }
    }
}
