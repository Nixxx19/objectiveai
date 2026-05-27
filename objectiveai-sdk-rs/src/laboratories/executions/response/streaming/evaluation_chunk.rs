use crate::{agent, functions};
use crate::agent::completions::response::streaming::AgentCompletionIds;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Streaming chunk for a single evaluation agent completion within a laboratory execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, JsonSchema, arbitrary::Arbitrary)]
#[schemars(rename = "laboratories.executions.response.streaming.EvaluationChunk")]
pub struct EvaluationChunk {
    /// Evaluation index (0-based).
    #[arbitrary(with = crate::arbitrary_util::arbitrary_u64)]
    pub index: u64,
    /// Agent index (0-based).
    #[arbitrary(with = crate::arbitrary_util::arbitrary_u64)]
    pub agent_index: u64,
    #[serde(flatten)]
    pub inner: agent::completions::response::streaming::AgentCompletionChunk,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub output: Option<functions::expression::InputValue>,
}

impl AgentCompletionIds for EvaluationChunk {
    fn agent_completion_ids(&self) -> impl Iterator<Item = &str> {
        self.inner.agent_completion_ids()
    }
}

impl EvaluationChunk {
    pub fn push(&mut self, other: &EvaluationChunk) {
        self.inner.push(&other.inner);
        if let Some(output) = &other.output {
            self.output = Some(output.clone());
        }
    }

    /// Produces log files for this evaluation completion.
    ///
    /// Returns `(reference, files)` where `reference` is a
    /// [`super::evaluation_log_reference::LogReference`] carrying
    /// `index`, `agent_index`, and optionally `output`. Files are
    /// written under `agent/completions/`.
    #[cfg(feature = "filesystem")]
    pub fn produce_files(
        &self,
    ) -> (super::evaluation_log_reference::LogReference, Vec<crate::filesystem::logs::LogFile>) {
        let (path, files) = match self.inner.produce_files() {
            Some((inner_ref, files)) => (inner_ref.path, files),
            None => (String::new(), Vec::new()),
        };
        let mut reference = super::evaluation_log_reference::LogReference::new(
            path,
            self.index,
            self.agent_index,
        );
        if let Some(output) = &self.output {
            reference.output = Some(serde_json::to_value(output).unwrap());
        }
        (reference, files)
    }

    /// Delegates to the inner agent completion's message-row extractor.
    #[cfg(feature = "filesystem")]
    pub fn produce_message_rows(
        &self,
    ) -> impl Iterator<Item = crate::filesystem::logs::queue::schema::MessageRow> + Send + '_ {
        self.inner.produce_message_rows()
    }
}
