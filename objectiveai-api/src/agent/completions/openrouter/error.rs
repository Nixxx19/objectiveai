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
}
