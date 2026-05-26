use crate::agent::completions::response::streaming::AgentCompletionIds;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, arbitrary::Arbitrary)]
#[schemars(rename = "functions.executions.response.streaming.FunctionExecutionTaskChunk")]
pub struct FunctionExecutionTaskChunk {
    #[arbitrary(with = crate::arbitrary_util::arbitrary_u64)]
    pub index: u64,
    #[arbitrary(with = crate::arbitrary_util::arbitrary_u64)]
    pub task_index: u64,
    #[arbitrary(with = crate::arbitrary_util::arbitrary_vec_u64)]
    pub task_path: Vec<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    #[arbitrary(with = crate::arbitrary_util::arbitrary_option_u64)]
    pub swiss_pool_index: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    #[arbitrary(with = crate::arbitrary_util::arbitrary_option_u64)]
    pub swiss_round: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    #[arbitrary(with = crate::arbitrary_util::arbitrary_option_u64)]
    pub split_index: Option<u64>,
    #[serde(flatten)]
    pub inner: super::FunctionExecutionChunk,
}

impl AgentCompletionIds for FunctionExecutionTaskChunk {
    fn agent_completion_ids(&self) -> impl Iterator<Item = &str> {
        self.inner.agent_completion_ids()
    }
}

impl FunctionExecutionTaskChunk {
    pub fn push(&mut self, other: &super::FunctionExecutionTaskChunk) {
        self.inner.push(&other.inner);
    }

    /// Produces log files for this nested function execution task.
    ///
    /// Returns `(reference, files)` where `reference` carries
    /// `index`, `task_index`, `task_path`, and optionally
    /// `swiss_pool_index`, `swiss_round`, `split_index`.
    /// Files under `functions/executions/`.
    #[cfg(feature = "filesystem")]
    pub fn produce_files(
        &self,
    ) -> (crate::filesystem::logs::LogReference, Vec<crate::filesystem::logs::LogFile>) {
        use crate::filesystem::logs::LogReference;
        let mut reference = match self.inner.produce_files() {
            Some((reference, files)) => {
                let mut r = reference;
                r.index = Some(self.index);
                r.task_index = Some(self.task_index);
                r.task_path = Some(self.task_path.clone());
                r.swiss_pool_index = self.swiss_pool_index;
                r.swiss_round = self.swiss_round;
                r.split_index = self.split_index;
                return (r, files);
            }
            None => LogReference::new(String::new()),
        };
        reference.index = Some(self.index);
        reference.task_index = Some(self.task_index);
        reference.task_path = Some(self.task_path.clone());
        reference.swiss_pool_index = self.swiss_pool_index;
        reference.swiss_round = self.swiss_round;
        reference.split_index = self.split_index;
        (reference, Vec::new())
    }
}
