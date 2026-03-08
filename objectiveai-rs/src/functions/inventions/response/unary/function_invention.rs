use crate::functions::inventions::response;
use crate::{agent, error, functions};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInvention {
    pub id: String,
    pub completions: Vec<super::AgentCompletion>,
    pub state: functions::inventions::State,
    pub function: Option<functions::FullRemoteFunction>,
    pub created: u64,
    pub object: super::Object,
    pub usage: agent::completions::response::Usage,
    pub error: Option<error::ResponseError>,
}

impl From<response::streaming::FunctionInventionChunk> for FunctionInvention {
    fn from(
        response::streaming::FunctionInventionChunk {
            id,
            completions,
            state,
            function,
            created,
            object,
            usage,
            error,
        }: response::streaming::FunctionInventionChunk,
    ) -> Self {
        Self {
            id,
            completions: completions
                .into_iter()
                .map(super::AgentCompletion::from)
                .collect(),
            state: state.unwrap_or(
                functions::inventions::State::AlphaScalarBranch(
                    functions::inventions::AlphaScalarBranchState {
                        params: functions::inventions::Params {
                            depth: 0,
                            min_branch_width: 0,
                            max_branch_width: 0,
                            min_leaf_width: 0,
                            max_leaf_width: 0,
                            name: String::new(),
                            spec: String::new(),
                        },
                        essay: None,
                        input_schema: None,
                        essay_tasks: None,
                        tasks: None,
                        tasks_length: None,
                        description: None,
                        readme: None,
                    },
                ),
            ),
            function,
            created,
            object: object.into(),
            usage: usage.unwrap_or_default(),
            error,
        }
    }
}
