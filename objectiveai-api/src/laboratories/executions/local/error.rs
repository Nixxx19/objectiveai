/// Errors that can occur during laboratory execution.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Docker operation failed.
    #[error("docker error: {0}")]
    Docker(String),
    /// MCP communication error.
    #[error("mcp error: {0}")]
    Mcp(String),
    /// No builder agents provided.
    #[error("at least one builder agent is required")]
    NoBuilderAgents,
    /// Agent completion failed.
    #[error("agent completion error: {0}")]
    AgentCompletion(String),
}

impl objectiveai::error::StatusError for Error {
    fn status(&self) -> u16 {
        match self {
            Error::Docker(_) => 500,
            Error::Mcp(_) => 500,
            Error::NoBuilderAgents => 400,
            Error::AgentCompletion(_) => 502,
        }
    }

    fn message(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "kind": "laboratory",
            "error": match self {
                Error::Docker(msg) => serde_json::json!({
                    "kind": "docker",
                    "error": msg,
                }),
                Error::Mcp(msg) => serde_json::json!({
                    "kind": "mcp",
                    "error": msg,
                }),
                Error::NoBuilderAgents => serde_json::json!({
                    "kind": "no_builder_agents",
                    "error": "at least one builder agent is required",
                }),
                Error::AgentCompletion(msg) => serde_json::json!({
                    "kind": "agent_completion",
                    "error": msg,
                }),
            }
        }))
    }
}
