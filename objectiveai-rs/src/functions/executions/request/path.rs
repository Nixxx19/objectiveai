//! Path parameters for function execution requests.
//!
//! These specify the remote source and repository references for remote
//! Functions and Profiles.

use crate::functions::Remote;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Path parameters for remote Function with inline Profile.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "functions.executions.request.FunctionRemoteProfileInlineRequestPath")]
pub struct FunctionRemoteProfileInlineRequestPath {
    /// Function remote source.
    pub fremote: Remote,
    /// Function repository owner.
    pub fowner: String,
    /// Function repository name.
    pub frepository: String,
    /// Function Git commit SHA (optional).
    pub fcommit: Option<String>,
}

/// Path parameters for inline Function with remote Profile.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "functions.executions.request.FunctionInlineProfileRemoteRequestPath")]
pub struct FunctionInlineProfileRemoteRequestPath {
    /// Profile remote source.
    pub premote: Remote,
    /// Profile repository owner.
    pub powner: String,
    /// Profile repository name.
    pub prepository: String,
    /// Profile Git commit SHA (optional).
    pub pcommit: Option<String>,
}

/// Path parameters for remote Function with remote Profile.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "functions.executions.request.FunctionRemoteProfileRemoteRequestPath")]
pub struct FunctionRemoteProfileRemoteRequestPath {
    /// Function remote source.
    pub fremote: Remote,
    /// Function repository owner.
    pub fowner: String,
    /// Function repository name.
    pub frepository: String,
    /// Function Git commit SHA (optional).
    pub fcommit: Option<String>,
    /// Profile remote source.
    pub premote: Remote,
    /// Profile repository owner.
    pub powner: String,
    /// Profile repository name.
    pub prepository: String,
    /// Profile Git commit SHA (optional).
    pub pcommit: Option<String>,
}
