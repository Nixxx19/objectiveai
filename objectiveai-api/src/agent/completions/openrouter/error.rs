//! Errors for OpenRouter agent completions.

/// Errors that can occur during OpenRouter agent completion construction.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An MCP server returned an error when listing tools.
    #[error("MCP error ({url}): {message}")]
    Mcp {
        /// The URL of the MCP server that errored.
        url: String,
        /// The error message from the MCP server.
        message: String,
    },
}
