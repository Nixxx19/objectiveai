//! HTTP MCP server configuration the Rust client passes to the Codex
//! runner over stdio. Codex's `Thread` API only consumes HTTP MCP
//! servers, so this is a single struct (no Stdio/SSE variants).
//!
//! The runner currently ignores `mcp_servers` — wiring this into
//! `Codex.Thread` is a follow-up. The wire shape is stable so callers
//! can start including it now.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// HTTP MCP server config — URL plus optional headers.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpServerConfig {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<IndexMap<String, String>>,
}

impl From<&objectiveai::mcp::Connection> for McpServerConfig {
    fn from(conn: &objectiveai::mcp::Connection) -> Self {
        let mut headers = IndexMap::new();

        if !conn.session_id.is_empty() {
            headers.insert("Mcp-Session-Id".to_string(), conn.session_id.clone());
        }
        if let Some(auth) = &conn.authorization {
            headers.insert("Authorization".to_string(), auth.clone());
        }
        if !conn.user_agent.is_empty() {
            headers.insert("User-Agent".to_string(), conn.user_agent.clone());
        }
        if !conn.x_title.is_empty() {
            headers.insert("X-Title".to_string(), conn.x_title.clone());
        }
        if !conn.http_referer.is_empty() {
            headers.insert("Referer".to_string(), conn.http_referer.clone());
            headers.insert("HTTP-Referer".to_string(), conn.http_referer.clone());
        }

        McpServerConfig {
            url: conn.url.clone(),
            headers: if headers.is_empty() { None } else { Some(headers) },
        }
    }
}
