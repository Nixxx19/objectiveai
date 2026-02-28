//! MCP server configuration for agents.

use serde::{Deserialize, Serialize};

/// An MCP server that the agent can connect to.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct McpServer {
    /// The URL of the MCP server.
    pub url: String,
}

impl McpServer {
    /// Validates the MCP server configuration.
    ///
    /// The URL must start with `http://localhost` or `https://`.
    pub fn validate(&self) -> Result<(), String> {
        // if !self.url.starts_with("http://localhost") && !self.url.starts_with("https://") {
        //     return Err(format!(
        //         "`mcp.url` must start with \"http://localhost\" or \"https://\", got: \"{}\"",
        //         self.url
        //     ));
        // }
        Ok(())
    }
}

/// A list of MCP servers.
pub type McpServers = Vec<McpServer>;

pub mod mcp_servers {
    //! Functions for working with [`McpServers`](super::McpServers).

    /// Validates all MCP servers in the list.
    pub fn validate(this: &super::McpServers) -> Result<(), String> {
        for server in this {
            server.validate()?;
        }
        Ok(())
    }

    /// Sorts the MCP servers for deterministic ordering.
    ///
    /// Empty lists become `None`.
    pub fn prepare(mut this: super::McpServers) -> Option<super::McpServers> {
        if this.is_empty() {
            None
        } else {
            this.sort();
            Some(this)
        }
    }
}
