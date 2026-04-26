//! Session manager.
//!
//! Holds the live MCP upstream connections and per-session routing state
//! that belong to each MCP session. Session IDs are UUIDv4s — 36 ASCII
//! visible characters (all in the 0x21-0x7E range required by MCP
//! 2025-06-18 §basic/transports#session-management).

use std::sync::Arc;

use dashmap::DashMap;
use futures::future::join_all;
use indexmap::IndexMap;
use objectiveai::mcp::{
    Connection,
    resource::{ListResourcesResult, ReadResourceResult, Resource},
    tool::{CallToolRequestParams, CallToolResult, ListToolsResult, Tool},
};
use tokio::sync::broadcast;

/// SSE event broadcast bound. The proxy may eventually publish list-changed
/// notifications; until then nothing is sent and the receiver just stays
/// open. Bound is small because there's nothing to back up.
const OUTBOUND_BROADCAST_CAPACITY: usize = 64;

/// Per-session state owned by the [`SessionManager`].
///
/// All routing, fan-out, and forwarding methods live here — `SessionManager`
/// is just the registry that hands out `Arc<Session>`s by id.
#[derive(Debug)]
pub struct Session {
    /// Live upstream MCP connections keyed by their
    /// `initialize_result.server_info.name`. The key is the same string the
    /// proxy uses as the `<server-name>_` prefix on every tool name and
    /// resource URI it ships, so routing inbound `tools/call` /
    /// `resources/read` is just a longest-prefix-match lookup against this
    /// map's keys — no side-channel cache to keep coherent.
    ///
    /// Insertion order matches the order URLs appeared in `X-MCP-Servers`,
    /// so listings are deterministic.
    pub connections: IndexMap<String, Arc<Connection>>,
    /// Fan-out channel for server-initiated SSE messages. The GET endpoint
    /// subscribes; future notification-forwarding code will publish into it.
    pub outbound: broadcast::Sender<serde_json::Value>,
}

impl Session {
    fn new(connections: IndexMap<String, Arc<Connection>>) -> Self {
        let (outbound, _) = broadcast::channel(OUTBOUND_BROADCAST_CAPACITY);
        Self {
            connections,
            outbound,
        }
    }

    /// Fan `tools/list` out to every upstream in parallel, prefix each
    /// tool's name with `<server-name>_`, concatenate the per-upstream
    /// lists, and return the union. Per-upstream failures are logged and
    /// the upstream is dropped from the result — one bad server can't
    /// poison the whole listing.
    pub async fn list_tools(&self) -> ListToolsResult {
        let names: Vec<&String> = self.connections.keys().collect();
        let results = join_all(
            self.connections
                .values()
                .map(|c| async move { c.list_tools().await }),
        )
        .await;

        let mut tools: Vec<Tool> = Vec::new();
        for (server_name, result) in names.into_iter().zip(results) {
            match result {
                Ok(arc) => {
                    for tool in arc.iter() {
                        let mut prefixed = tool.clone();
                        prefixed.name = prefix_name(server_name, &tool.name);
                        tools.push(prefixed);
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, upstream = %server_name, "list_tools failed")
                }
            }
        }

        ListToolsResult {
            tools,
            next_cursor: None,
            _meta: None,
        }
    }

    /// Fan `resources/list` out to every upstream in parallel, prefix each
    /// URI with `<server-name>_`, concatenate the per-upstream lists, and
    /// return the union. Same best-effort failure semantics as
    /// [`Session::list_tools`].
    pub async fn list_resources(&self) -> ListResourcesResult {
        let names: Vec<&String> = self.connections.keys().collect();
        let results = join_all(
            self.connections
                .values()
                .map(|c| async move { c.list_resources().await }),
        )
        .await;

        let mut resources: Vec<Resource> = Vec::new();
        for (server_name, result) in names.into_iter().zip(results) {
            match result {
                Ok(arc) => {
                    for resource in arc.iter() {
                        let mut prefixed = resource.clone();
                        prefixed.uri = prefix_name(server_name, &resource.uri);
                        resources.push(prefixed);
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, upstream = %server_name, "list_resources failed")
                }
            }
        }

        ListResourcesResult {
            resources,
            next_cursor: None,
            _meta: None,
        }
    }

    /// Forward `tools/call` to whichever upstream owns the named tool.
    /// Routing is longest-prefix-match against the connection map's keys —
    /// see [`Session::route`].
    pub async fn call_tool(
        &self,
        params: &CallToolRequestParams,
    ) -> Result<CallToolResult, CallToolError> {
        let (connection, original_name) = self
            .route(&params.name)
            .ok_or_else(|| CallToolError::ToolNotFound(params.name.clone()))?;

        // Forward to the upstream with the un-prefixed tool name it actually
        // knows; pass everything else (`arguments`, `task`, `_meta`) through
        // unchanged.
        let upstream_params = CallToolRequestParams {
            name: original_name,
            arguments: params.arguments.clone(),
            task: params.task.clone(),
            _meta: params._meta.clone(),
        };
        Ok(connection.call_tool(&upstream_params).await?)
    }

    /// Forward `resources/read` to whichever upstream owns the URI. Same
    /// longest-prefix-match routing as [`Session::call_tool`].
    pub async fn read_resource(
        &self,
        uri: &str,
    ) -> Result<ReadResourceResult, ReadResourceError> {
        let (connection, original_uri) = self
            .route(uri)
            .ok_or_else(|| ReadResourceError::ResourceNotFound(uri.to_string()))?;
        Ok(connection.read_resource(&original_uri).await?)
    }

    /// Resolve a `<server-name>_<original>` prefixed identifier to the
    /// owning connection and the original (un-prefixed) name the upstream
    /// actually knows.
    ///
    /// Server names that contain `_` are supported via longest-prefix
    /// match: if both `fs` and `fs_extra` are connected and the inbound
    /// name is `fs_extra_Read`, the `fs_extra` upstream wins.
    fn route<'a>(&'a self, prefixed: &str) -> Option<(&'a Arc<Connection>, String)> {
        let mut best: Option<(&'a str, &'a Arc<Connection>)> = None;
        for (name, conn) in &self.connections {
            // Need at least one char after the `_` to count as a real prefix
            // hit (otherwise an exact match `name == prefixed` would route
            // to an empty original name).
            if prefixed.len() > name.len() + 1
                && prefixed.as_bytes()[name.len()] == b'_'
                && prefixed.starts_with(name.as_str())
            {
                if best.map(|(b, _)| name.len() > b.len()).unwrap_or(true) {
                    best = Some((name.as_str(), conn));
                }
            }
        }
        best.map(|(name, conn)| {
            let original = prefixed[name.len() + 1..].to_string();
            (conn, original)
        })
    }
}

/// Prefix a tool name or resource URI with the upstream server name.
/// Format: `<server-name>_<original>`.
fn prefix_name(server_name: &str, name: &str) -> String {
    format!("{server_name}_{name}")
}

/// Failure modes for [`Session::call_tool`].
#[derive(Debug, thiserror::Error)]
pub enum CallToolError {
    #[error("tool not found on any upstream: {0}")]
    ToolNotFound(String),
    #[error("upstream call_tool failed: {0}")]
    Upstream(#[from] objectiveai::mcp::Error),
}

/// Failure modes for [`Session::read_resource`].
#[derive(Debug, thiserror::Error)]
pub enum ReadResourceError {
    #[error("resource not found on any upstream: {0}")]
    ResourceNotFound(String),
    #[error("upstream read_resource failed: {0}")]
    Upstream(#[from] objectiveai::mcp::Error),
}

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
        let mut by_name: IndexMap<String, Arc<Connection>> = IndexMap::with_capacity(connections.len());
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
}
