use objectiveai::error::StatusError;

/// Errors that can occur during Function invention.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error from agent completions.
    #[error("agent completions error: {0}")]
    AgentCompletions(#[from] crate::agent::completions::Error),
    /// The invention state is invalid.
    #[error("invalid state: {0}")]
    InvalidState(String),
    /// The name already exists and overwrite is not enabled.
    #[error("name already exists: {0}")]
    NameAlreadyExists(String),
    /// GitHub token is missing when remote is GitHub.
    #[error("github token required")]
    GithubTokenRequired,
    /// GitHub token validation failed.
    #[error("github token error: {0}")]
    GithubToken(#[from] crate::github::Error),
    /// GitHub token lacks required permissions.
    #[error("github token missing permissions: {0}")]
    GithubTokenMissingPermissions(String),
    /// The name is invalid (too long or would exceed limits with child paths).
    #[error("invalid name: {0}")]
    InvalidName(String),
    /// Filesystem error.
    #[error("filesystem error: {0}")]
    Filesystem(#[from] crate::filesystem::Error),
}

impl StatusError for Error {
    fn status(&self) -> u16 {
        match self {
            Error::AgentCompletions(e) => e.status(),
            Error::InvalidState(_) => 400,
            Error::NameAlreadyExists(_) => 409,
            Error::GithubTokenRequired => 400,
            Error::GithubToken(e) => e.status(),
            Error::GithubTokenMissingPermissions(_) => 403,
            Error::InvalidName(_) => 400,
            Error::Filesystem(e) => e.status(),
        }
    }

    fn message(&self) -> Option<serde_json::Value> {
        let error_value = match self {
            Error::AgentCompletions(e) => serde_json::json!({
                "kind": "agent_completions",
                "error": e.message(),
            }),
            Error::InvalidState(msg) => serde_json::json!({
                "kind": "invalid_state",
                "error": msg,
            }),
            Error::NameAlreadyExists(name) => serde_json::json!({
                "kind": "name_already_exists",
                "error": format!("Repository '{}' already exists. Set overwrite to true to allow this.", name),
            }),
            Error::GithubTokenRequired => serde_json::json!({
                "kind": "github_token_required",
                "error": "A github_token is required when remote is GitHub.",
            }),
            Error::GithubToken(e) => serde_json::json!({
                "kind": "github_token",
                "error": e.message(),
            }),
            Error::GithubTokenMissingPermissions(msg) => serde_json::json!({
                "kind": "github_token_missing_permissions",
                "error": msg,
            }),
            Error::InvalidName(msg) => serde_json::json!({
                "kind": "invalid_name",
                "error": msg,
            }),
            Error::Filesystem(e) => serde_json::json!({
                "kind": "filesystem",
                "error": e.message(),
            }),
        };
        Some(serde_json::json!({
            "kind": "invention",
            "error": error_value,
        }))
    }
}
