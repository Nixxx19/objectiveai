/// Errors that can occur during Function invention.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error from an upstream provider.
    #[error("upstream error: {0}")]
    Upstream(#[from] super::upstream::Error),
    /// The invention state is invalid.
    #[error("invalid state: {0}")]
    InvalidState(String),
}

impl objectiveai::error::StatusError for Error {
    fn status(&self) -> u16 {
        match self {
            Error::Upstream(e) => e.status(),
            Error::InvalidState(_) => 400,
        }
    }

    fn message(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "kind": "invention",
            "error": match self {
                Error::Upstream(e) => serde_json::json!({
                    "kind": "upstream",
                    "error": e.message(),
                }),
                Error::InvalidState(msg) => serde_json::json!({
                    "kind": "invalid_state",
                    "error": msg,
                }),
            }
        }))
    }
}
