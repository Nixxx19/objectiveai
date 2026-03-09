use objectiveai::error::StatusError;

/// Errors that can occur during recursive Function invention.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Error from the single-level invention client.
    #[error("invention error: {0}")]
    Invention(#[from] crate::functions::inventions::Error),
}

impl StatusError for Error {
    fn status(&self) -> u16 {
        match self {
            Error::Invention(e) => e.status(),
        }
    }

    fn message(&self) -> Option<serde_json::Value> {
        let error_value = match self {
            Error::Invention(e) => serde_json::json!({
                "kind": "invention",
                "error": e.message(),
            }),
        };
        Some(serde_json::json!({
            "kind": "recursive_invention",
            "error": error_value,
        }))
    }
}
