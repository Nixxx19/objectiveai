//! Remote source types for function and profile hosting.

use serde::{Deserialize, Serialize};
use std::fmt;

/// The remote source where a function or profile is hosted.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum Remote {
    /// GitHub repository.
    Github,
    /// Local filesystem.
    Filesystem,
}

impl Remote {
    pub fn url(&self, owner: &str, repository: &str, commit: &str) -> String {
        match self {
            Remote::Github => format!(
                "https://github.com/{}/{}/commit/{}",
                owner, repository, commit
            ),
            Remote::Filesystem => {
                format!("file://{}/{} ({})", owner, repository, commit)
            }
        }
    }
}

impl fmt::Display for Remote {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Remote::Github => write!(f, "github"),
            Remote::Filesystem => write!(f, "filesystem"),
        }
    }
}
