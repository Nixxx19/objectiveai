#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid agent: {0}")]
    InvalidAgent(String),

    #[error("agent not found: {0}")]
    AgentNotFound(String),

    #[error("MCP connection error: {0}")]
    McpConnection(#[from] crate::mcp::Error),

    #[error("{0}")]
    Fetch(objectiveai::error::ResponseError),
}

impl objectiveai::error::StatusError for Error {
    fn status(&self) -> u16 {
        match self {
            Self::InvalidAgent(_) => 400,
            Self::AgentNotFound(_) => 404,
            Self::McpConnection(_) => 502,
            Self::Fetch(e) => e.code,
        }
    }

    fn message(&self) -> Option<serde_json::Value> {
        match self {
            Self::Fetch(e) => Some(e.message.clone()),
            _ => Some(serde_json::Value::String(self.to_string())),
        }
    }
}

impl From<objectiveai::error::ResponseError> for Error {
    fn from(e: objectiveai::error::ResponseError) -> Self {
        Self::Fetch(e)
    }
}
