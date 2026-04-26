//! Session registry.
//!
//! Maps session ids to [`Session`]s. Session IDs are UUIDv4s — 36 ASCII
//! visible characters (all in the 0x21-0x7E range required by MCP
//! 2025-06-18 §basic/transports#session-management). All per-session
//! dispatch (list, call, read) lives on [`Session`] itself; this file only
//! cares about minting ids, packing connections into a [`Session`], and
//! looking sessions back up.

use std::sync::Arc;

use dashmap::DashMap;

use crate::session::Session;

/// Maps a session id to its [`Session`] state.
#[derive(Debug, Default)]
pub struct SessionManager {
    sessions: DashMap<String, Arc<Session>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an already-constructed [`Session`] and return its
    /// freshly-minted session id.
    pub fn add(&self, session: Session) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        self.sessions.insert(id.clone(), Arc::new(session));
        id
    }

    /// Cheap clone-out of a [`Session`] — never holds a DashMap guard
    /// across the await boundary.
    pub fn get(&self, session_id: &str) -> Option<Arc<Session>> {
        self.sessions.get(session_id).map(|e| e.value().clone())
    }
}
