//! Remote source types for function, profile, and agent hosting.

use serde::{Deserialize, Serialize};
use std::fmt;
use schemars::JsonSchema;

/// The remote source where a function, profile, or agent is hosted.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, JsonSchema, arbitrary::Arbitrary)]
#[serde(rename_all = "snake_case")]
#[schemars(rename = "Remote")]
pub enum Remote {
    /// GitHub repository.
    Github,
    /// Local filesystem.
    Filesystem,
    /// Mock (for testing).
    Mock,
}

impl Remote {
    pub fn url(&self, owner: &str, repository: &str, commit: &str) -> String {
        match self {
            Remote::Github => format!(
                "[{}](https://github.com/{}/{}/commit/{})",
                repository, owner, repository, commit
            ),
            Remote::Filesystem => {
                format!(
                    "[{}](file://{}/{}) ({})",
                    repository, owner, repository, commit
                )
            }
            Remote::Mock => {
                format!("[{}](mock://{}/{}) ({})", repository, owner, repository, commit)
            }
        }
    }
}

impl fmt::Display for Remote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Remote::Github => write!(f, "github"),
            Remote::Filesystem => write!(f, "filesystem"),
            Remote::Mock => write!(f, "mock"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, JsonSchema, arbitrary::Arbitrary)]
#[schemars(rename = "RemotePath")]
pub struct RemotePath {
    pub remote: Remote,
    pub owner: String,
    pub repository: String,
    pub commit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, JsonSchema, arbitrary::Arbitrary)]
#[schemars(rename = "RemotePathCommitOptional")]
pub struct RemotePathCommitOptional {
    pub remote: Remote,
    pub owner: String,
    pub repository: String,
    pub commit: Option<String>,
}
