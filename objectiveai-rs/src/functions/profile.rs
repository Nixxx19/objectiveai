//! Profile types for Function execution.
//!
//! A Profile contains the learned weights and configuration needed to execute
//! a Function. Profiles are typically trained on example data to optimize
//! scoring behavior.

use crate::vector;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// A Profile definition, either remote or inline.
///
/// Profiles contain the weights and nested configurations needed to execute
/// a Function. They correspond to a Function's task structure.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(rename = "functions.Profile")]
pub enum Profile {
    /// A remote profile with metadata.
    Remote(RemoteProfile),
    /// An inline profile definition.
    Inline(InlineProfile),
}

/// A remote profile, either tasks-based or auto.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(rename = "functions.RemoteProfile")]
pub enum RemoteProfile {
    /// Tasks-based profile with per-task configuration.
    Tasks(RemoteTasksProfile),
    /// Auto profile that applies a single swarm+weights to all vector completion tasks.
    Auto(RemoteAutoProfile),
}

/// An inline profile, either tasks-based or auto.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(rename = "functions.InlineProfile")]
pub enum InlineProfile {
    /// Tasks-based profile with per-task configuration.
    Tasks(InlineTasksProfile),
    /// Auto profile that applies a single swarm+weights to all vector completion tasks.
    Auto(InlineAutoProfile),
}

impl<'a> arbitrary::Arbitrary<'a> for InlineProfile {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        if u.arbitrary().unwrap_or(false) {
            Ok(InlineProfile::Tasks(u.arbitrary()?))
        } else {
            Ok(InlineProfile::Auto(u.arbitrary()?))
        }
    }
}

/// A remote tasks-based profile with full metadata.
///
/// Stored as `profile.json` in repositories and referenced by
/// `remote/owner/repository`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "functions.RemoteTasksProfile")]
pub struct RemoteTasksProfile {
    /// Human-readable description of the profile.
    pub description: String,
    /// Configuration for each task in the corresponding Function.
    pub tasks: Vec<TaskProfile>,
    /// Weights for each Task in the corresponding Function.
    ///
    /// Must have the same length as `tasks`. Can be either:
    /// - A vector of decimals (legacy representation), or
    /// - A vector of objects with `weight` and optional `invert` fields.
    pub profile: crate::vector::completions::request::Profile,
}

/// A remote auto profile with full metadata.
///
/// Applies a single swarm and weights to every vector completion task
/// in the function, with equal task weights.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "functions.RemoteAutoProfile")]
pub struct RemoteAutoProfile {
    /// Human-readable description of the profile.
    pub description: String,
    /// The swarm to use for all vector completion tasks.
    pub swarm: vector::completions::request::Swarm,
    /// Weights for each agent in the swarm.
    pub profile: crate::vector::completions::request::Profile,
}

/// An inline tasks-based profile definition without metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, arbitrary::Arbitrary)]
#[schemars(rename = "functions.InlineTasksProfile")]
pub struct InlineTasksProfile {
    /// Configuration for each task in the corresponding Function.
    pub tasks: Vec<TaskProfile>,
    /// Weights for each Task in the corresponding Function.
    ///
    /// Must have the same length as `tasks`. Can be either:
    /// - A vector of decimals (legacy representation), or
    /// - A vector of objects with `weight` and optional `invert` fields.
    pub profile: crate::vector::completions::request::Profile,
}

/// An inline auto profile definition without metadata.
///
/// Applies a single swarm and weights to every vector completion task
/// in the function, with equal task weights.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, arbitrary::Arbitrary)]
#[schemars(rename = "functions.InlineAutoProfile")]
pub struct InlineAutoProfile {
    /// The swarm to use for all vector completion tasks.
    pub swarm: vector::completions::request::Swarm,
    /// Weights for each agent in the swarm.
    pub profile: crate::vector::completions::request::Profile,
}

/// Configuration for a single task within a Profile.
///
/// Each variant corresponds to a task type in the Function definition.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
#[schemars(rename = "functions.TaskProfile")]
pub enum TaskProfile {
    /// Profile for a nested function task (references another profile).
    Remote {
        /// The remote source where the profile is hosted.
        remote: super::Remote,
        /// Repository owner.
        owner: String,
        /// Repository name.
        repository: String,
        /// Git commit SHA. Highly recommended for remote profiles to
        /// ensure compatibility if the referenced profile's shape changes.
        commit: Option<String>,
    },
    /// Inline profile for a task (tasks-based or auto).
    Inline(InlineProfile),
    /// Placeholder task — no configuration needed, output is fixed.
    Placeholder {},
}

impl<'a> arbitrary::Arbitrary<'a> for TaskProfile {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        // Inline variant is recursive (InlineProfile → InlineTasksProfile → Vec<TaskProfile>),
        // so use go-deep bool to make recursion exponentially rare.
        if u.arbitrary().unwrap_or(false) {
            Ok(TaskProfile::Inline(u.arbitrary()?))
        } else if u.arbitrary().unwrap_or(false) {
            Ok(TaskProfile::Remote {
                remote: u.arbitrary()?,
                owner: u.arbitrary()?,
                repository: u.arbitrary()?,
                commit: u.arbitrary()?,
            })
        } else {
            Ok(TaskProfile::Placeholder {})
        }
    }
}

impl TaskProfile {
    /// Returns `false` if any remote function reference is missing a commit SHA.
    ///
    /// While not strictly required, specifying commit SHAs is highly recommended
    /// for remote profiles to ensure the profile remains valid if the
    /// referenced profile's structure changes.
    pub fn validate_commit_required(&self) -> bool {
        match self {
            TaskProfile::Remote { commit, .. } => commit.is_some(),
            TaskProfile::Inline(InlineProfile::Tasks(inline)) => inline
                .tasks
                .iter()
                .all(TaskProfile::validate_commit_required),
            TaskProfile::Inline(InlineProfile::Auto(_)) => true,
            TaskProfile::Placeholder {} => true,
        }
    }
}
