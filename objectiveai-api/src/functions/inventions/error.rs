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
}

impl StatusError for Error {
    fn status(&self) -> u16 {
        match self {
            Error::AgentCompletions(e) => e.status(),
            Error::InvalidState(_) => 400,
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
        };
        Some(serde_json::json!({
            "kind": "invention",
            "error": error_value,
        }))
    }
}
