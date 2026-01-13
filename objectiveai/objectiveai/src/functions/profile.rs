use crate::vector;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Profile {
    Remote(RemoteProfile),
    Inline(InlineProfile),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteProfile {
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changelog: Option<String>,
    pub tasks: Vec<TaskProfileCommitRequired>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineProfile {
    pub tasks: Vec<TaskProfileCommitOptional>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputedProfile {
    pub tasks: Vec<TaskProfileCommitRequired>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TaskProfile {
    CommitRequired(TaskProfileCommitRequired),
    CommitOptional(TaskProfileCommitOptional),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TaskProfileCommitRequired {
    RemoteFunction {
        owner: String,
        repository: String,
        commit: String,
    },
    InlineFunction(Vec<TaskProfileCommitRequired>),
    VectorCompletion {
        ensemble: vector::completions::request::Ensemble,
        profile: Vec<rust_decimal::Decimal>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TaskProfileCommitOptional {
    RemoteFunction {
        owner: String,
        repository: String,
        commit: Option<String>,
    },
    InlineFunction(Vec<TaskProfileCommitOptional>),
    VectorCompletion {
        ensemble: vector::completions::request::Ensemble,
        profile: Vec<rust_decimal::Decimal>,
    },
}

impl TaskProfileCommitOptional {
    pub fn try_into_commit_required(self) -> Option<TaskProfileCommitRequired> {
        match self {
            TaskProfileCommitOptional::RemoteFunction {
                owner,
                repository,
                commit: Some(commit),
            } => Some(TaskProfileCommitRequired::RemoteFunction {
                owner,
                repository,
                commit,
            }),
            TaskProfileCommitOptional::RemoteFunction {
                commit: None, ..
            } => None,
            TaskProfileCommitOptional::InlineFunction(tasks) => {
                let mut required_tasks = Vec::with_capacity(tasks.len());
                for task in tasks {
                    if let Some(required_task) = task.try_into_commit_required()
                    {
                        required_tasks.push(required_task);
                    } else {
                        return None;
                    }
                }
                Some(TaskProfileCommitRequired::InlineFunction(required_tasks))
            }
            TaskProfileCommitOptional::VectorCompletion {
                ensemble,
                profile,
            } => Some(TaskProfileCommitRequired::VectorCompletion {
                ensemble,
                profile,
            }),
        }
    }
}
