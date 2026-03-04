#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("rate limited")]
    RateLimit,

    #[error("invalid continuation: {0}")]
    InvalidContinuation(String),

    #[error("BYOK is not supported for Claude Agent SDK")]
    InvalidByok,

    #[error("invalid messages: {0}")]
    InvalidMessages(String),
}

impl objectiveai::error::StatusError for Error {
    fn status(&self) -> u16 {
        match self {
            Self::RateLimit => 429,
            Self::InvalidContinuation(_) => 400,
            Self::InvalidByok => 400,
            Self::InvalidMessages(_) => 400,
        }
    }

    fn message(&self) -> Option<serde_json::Value> {
        Some(serde_json::Value::String(self.to_string()))
    }
}
