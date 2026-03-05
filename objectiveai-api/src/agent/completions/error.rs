#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("invalid agent: {0}")]
    InvalidAgent(String),

    #[error("agent not found: {0}")]
    AgentNotFound(String),

    #[error("MCP connection error: {0}")]
    McpConnection(crate::mcp::Error),

    #[error("MCP list_tools error ({url}): {error}")]
    McpListTools {
        url: String,
        error: std::sync::Arc<crate::mcp::Error>,
    },

    #[error("MCP call_tool error: {0}")]
    McpCallTool(crate::mcp::Error),

    #[error("{0}")]
    Fetch(objectiveai::error::ResponseError),

    #[error("upstream error: {0}")]
    Upstream(objectiveai::error::ResponseError),

    #[error("no agents resolved")]
    NoAgentsResolved,

    #[error("all agents failed: {0:?}")]
    MultipleErrors(Vec<Error>),

    #[error("timeout")]
    Timeout,

    #[error("empty stream")]
    EmptyStream,
}

impl objectiveai::error::StatusError for Error {
    fn status(&self) -> u16 {
        match self {
            Self::InvalidAgent(_) => 400,
            Self::AgentNotFound(_) => 404,
            Self::McpConnection(_) => 502,
            Self::McpListTools { .. } => 502,
            Self::McpCallTool(_) => 502,
            Self::Fetch(e) => e.code,
            Self::Upstream(e) => e.code,
            Self::NoAgentsResolved => 400,
            Self::MultipleErrors(errors) => {
                errors.first().map(|e| e.status()).unwrap_or(500)
            }
            Self::Timeout => 504,
            Self::EmptyStream => 502,
        }
    }

    fn message(&self) -> Option<serde_json::Value> {
        match self {
            Self::Fetch(e) | Self::Upstream(e) => Some(e.message.clone()),
            _ => Some(serde_json::Value::String(self.to_string())),
        }
    }
}
