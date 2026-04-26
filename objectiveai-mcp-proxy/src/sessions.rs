//! Session manager.
//!
//! Holds the live MCP upstream connections that belong to each MCP session.
//! Session IDs are UUIDv4s — 36 ASCII visible characters (all in the
//! 0x21-0x7E range required by MCP 2025-06-18 §basic/transports#session-management).

use std::sync::Arc;

use dashmap::DashMap;
use futures::future::join_all;
use objectiveai::mcp::{
    Connection,
    resource::{ListResourcesResult, Resource},
    tool::{ListToolsResult, Tool},
};

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

    /// Snapshot the connections belonging to a session.
    ///
    /// Returns `None` if the session id is unknown. Cloning out the Vec
    /// before doing any awaits is required — DashMap's read guard is sync
    /// and would otherwise be held across await points.
    fn snapshot(&self, session_id: &str) -> Option<Vec<Arc<Connection>>> {
        self.sessions.get(session_id).map(|e| e.value().clone())
    }

    /// Fan `tools/list` out to every upstream in the session in parallel,
    /// concatenate the per-upstream tool lists, and return the union as a
    /// single `ListToolsResult`. Per-upstream failures are logged and the
    /// upstream is dropped from the result — the proxy is best-effort so
    /// one bad server can't poison the whole listing.
    pub async fn list_tools(&self, session_id: &str) -> Option<ListToolsResult> {
        let connections = self.snapshot(session_id)?;
        let results = join_all(
            connections
                .iter()
                .map(|c| async move { c.list_tools().await }),
        )
        .await;

        let mut tools: Vec<Tool> = Vec::new();
        for result in results {
            match result {
                Ok(arc) => tools.extend(arc.iter().cloned()),
                Err(e) => tracing::warn!(error = %e, "list_tools failed for upstream"),
            }
        }

        Some(ListToolsResult {
            tools,
            next_cursor: None,
            _meta: None,
        })
    }

    /// Fan `resources/list` out to every upstream in the session in
    /// parallel, concatenate the per-upstream resource lists, and return
    /// the union as a single `ListResourcesResult`. Same best-effort
    /// failure semantics as `list_tools`.
    pub async fn list_resources(&self, session_id: &str) -> Option<ListResourcesResult> {
        let connections = self.snapshot(session_id)?;
        let results = join_all(
            connections
                .iter()
                .map(|c| async move { c.list_resources().await }),
        )
        .await;

        let mut resources: Vec<Resource> = Vec::new();
        for result in results {
            match result {
                Ok(arc) => resources.extend(arc.iter().cloned()),
                Err(e) => tracing::warn!(error = %e, "list_resources failed for upstream"),
            }
        }

        Some(ListResourcesResult {
            resources,
            next_cursor: None,
            _meta: None,
        })
    }
}
