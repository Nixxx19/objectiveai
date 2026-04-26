//! Session manager.
//!
//! Holds the live MCP upstream connections that belong to each MCP session.
//! Session IDs are UUIDv4s — 36 ASCII visible characters (all in the
//! 0x21-0x7E range required by MCP 2025-06-18 §basic/transports#session-management).

use std::sync::Arc;

use dashmap::DashMap;
use objectiveai::mcp::Connection;

/// Maps a session id to the upstream MCP connections that belong to it.
#[derive(Debug, Default)]
pub struct SessionManager {
    sessions: DashMap<String, Vec<Arc<Connection>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new session and return its freshly-minted session id.
    pub fn add(&self, connections: Vec<Arc<Connection>>) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        self.sessions.insert(id.clone(), connections);
        id
    }
}
