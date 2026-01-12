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
