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
use indexmap::IndexMap;
use objectiveai::mcp::Connection;

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

    /// Register a new session and return its freshly-minted session id.
    ///
    /// Connections are keyed by their upstream `server_info.name`. If two
    /// upstreams advertise the same name, the later one wins with a warn —
    /// the proxy's prefix scheme can't disambiguate them anyway, so
    /// silently keeping both would create unroutable tools.
    pub fn add(&self, connections: Vec<Arc<Connection>>) -> String {
        let mut by_name: IndexMap<String, Arc<Connection>> =
            IndexMap::with_capacity(connections.len());
        for connection in connections {
            let name = connection.initialize_result.server_info.name.clone();
            if by_name.contains_key(&name) {
                tracing::warn!(
                    server_name = %name,
                    "two upstreams report the same server_info.name; later upstream wins",
                );
            }
            by_name.insert(name, connection);
        }

        let id = uuid::Uuid::new_v4().to_string();
        self.sessions
            .insert(id.clone(), Arc::new(Session::new(by_name)));
        id
    }

    /// Cheap clone-out of a [`Session`] — never holds a DashMap guard
    /// across the await boundary.
    pub fn get(&self, session_id: &str) -> Option<Arc<Session>> {
        self.sessions.get(session_id).map(|e| e.value().clone())
    }

    /// Remove a session from the registry. Returns `Some(_)` if a session
    /// was present, `None` if the id was unknown.
    ///
    /// Note: removing the session releases this map's `Arc<Session>`, but
    /// each `Arc<Connection>` inside the session has long-running
    /// background tasks (the upstream SSE listener) that hold their own
    /// `Arc<Connection>` clones — those tasks keep the upstream
    /// connections alive until the proxy process restarts. A proper shutdown
    /// signal on `Connection` is needed to fully reclaim the resources.
    pub fn remove(&self, session_id: &str) -> Option<Arc<Session>> {
        self.sessions.remove(session_id).map(|(_, session)| session)
    }
}
