//! Errors for OpenRouter agent completions.

use std::sync::Arc;

/// Errors that can occur during OpenRouter agent completion construction.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An MCP server returned an error when listing tools.
    #[error("MCP error ({url}): {error}")]
    Mcp {
        /// The URL of the MCP server that errored.
        url: String,
        /// The underlying MCP error.
        error: Arc<crate::mcp::Error>,
    },

    /// The upstream produced no chunks.
    #[error("empty stream")]
    EmptyStream,
}

impl objectiveai::error::StatusError for Error {
    fn status(&self) -> u16 {
        match self {
            Self::Mcp { .. } => 502,
            Self::EmptyStream => 500,
        }
    }

    fn message(&self) -> Option<serde_json::Value> {
        Some(serde_json::Value::String(self.to_string()))
    }
}
