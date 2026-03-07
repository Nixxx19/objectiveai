#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Expected error")]
    ExpectedError,

    #[error("unsupported response format: {0}")]
    UnsupportedResponseFormat(String),

    #[error("tools not allowed but response format requires a tool call")]
    ToolsNotAllowedWithRequiredToolCall,

    #[error("invention agent requires invention tools")]
    InventionAgentWithoutInventionTools,
}

impl objectiveai::error::StatusError for Error {
    fn status(&self) -> u16 {
        match self {
            Self::ExpectedError => 500,
            Self::UnsupportedResponseFormat(_) => 400,
            Self::ToolsNotAllowedWithRequiredToolCall => 400,
            Self::InventionAgentWithoutInventionTools => 400,
        }
    }

    fn message(&self) -> Option<serde_json::Value> {
        Some(serde_json::Value::String(self.to_string()))
    }
}
