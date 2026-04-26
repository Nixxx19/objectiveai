//! Session manager.
//!
//! Holds the live MCP upstream connections and per-session routing state
//! that belong to each MCP session. Session IDs are UUIDv4s — 36 ASCII
//! visible characters (all in the 0x21-0x7E range required by MCP
//! 2025-06-18 §basic/transports#session-management).

use std::sync::Arc;

use dashmap::DashMap;
use futures::future::join_all;
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
#[derive(Debug)]
pub struct Session {
    /// Live upstream MCP connections, in the order their URLs appeared in
    /// `X-MCP-Servers`.
    pub connections: Vec<Arc<Connection>>,
    /// Prefixed tool name → (index into `connections`, original tool name
    /// the upstream knows it by). Populated lazily by
    /// [`SessionManager::list_tools`] and refreshed on a cache miss inside
    /// [`SessionManager::call_tool`].
    tool_owner: DashMap<String, (usize, String)>,
    /// Prefixed resource URI → (index into `connections`, original URI the
    /// upstream knows it by). Same lifecycle as `tool_owner`.
    resource_owner: DashMap<String, (usize, String)>,
    /// Fan-out channel for server-initiated SSE messages. The GET
    /// endpoint subscribes; future notification-forwarding code will
    /// publish into it.
    pub outbound: broadcast::Sender<serde_json::Value>,
}

impl Session {
    fn new(connections: Vec<Arc<Connection>>) -> Self {
        let (outbound, _) = broadcast::channel(OUTBOUND_BROADCAST_CAPACITY);
        Self {
            connections,
            tool_owner: DashMap::new(),
            resource_owner: DashMap::new(),
            outbound,
        }
    }
}

/// Prefix a tool/resource name with the upstream server's
/// `initialize_result.server_info.name` so two upstreams that both expose
/// e.g. `search` don't collide. Format: `<server-name>_<original>`.
fn prefix_name(connection: &Connection, name: &str) -> String {
    format!(
        "{}_{}",
        connection.initialize_result.server_info.name,
        name
    )
}

/// Failure modes for [`SessionManager::call_tool`].
#[derive(Debug, thiserror::Error)]
pub enum CallToolError {
    #[error("session not found: {0}")]
    SessionNotFound(String),
    #[error("tool not found on any upstream: {0}")]
    ToolNotFound(String),
    #[error("upstream call_tool failed: {0}")]
    Upstream(#[from] objectiveai::mcp::Error),
}

/// Failure modes for [`SessionManager::read_resource`].
#[derive(Debug, thiserror::Error)]
pub enum ReadResourceError {
    #[error("session not found: {0}")]
    SessionNotFound(String),
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
    pub fn add(&self, connections: Vec<Arc<Connection>>) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        self.sessions.insert(id.clone(), Arc::new(Session::new(connections)));
        id
    }

    /// Cheap clone-out of a [`Session`] — never holds a DashMap guard
    /// across the await boundary.
    pub fn get(&self, session_id: &str) -> Option<Arc<Session>> {
        self.sessions.get(session_id).map(|e| e.value().clone())
    }

    /// Fan `tools/list` out to every upstream in the session in parallel,
    /// concatenate the per-upstream tool lists, populate the in-session
    /// tool→connection routing map as a side effect, and return the union
    /// as a single [`ListToolsResult`].
    ///
    /// Per-upstream failures are logged and the upstream is dropped from
    /// the result — one bad server can't poison the whole listing.
    pub async fn list_tools(&self, session_id: &str) -> Option<ListToolsResult> {
        let session = self.get(session_id)?;
        let results = join_all(
            session
                .connections
                .iter()
                .map(|c| async move { c.list_tools().await }),
        )
        .await;

        let mut tools: Vec<Tool> = Vec::new();
        for (idx, result) in results.into_iter().enumerate() {
            let connection = &session.connections[idx];
            match result {
                Ok(arc) => {
                    for tool in arc.iter() {
                        let prefixed = prefix_name(connection, &tool.name);
                        if let Some(prev) = session
                            .tool_owner
                            .insert(prefixed.clone(), (idx, tool.name.clone()))
                        {
                            if prev.0 != idx {
                                tracing::warn!(
                                    tool = %prefixed,
                                    previous_upstream = prev.0,
                                    new_upstream = idx,
                                    "prefixed tool name collision; new upstream wins",
                                );
                            }
                        }
                        let mut prefixed_tool = tool.clone();
                        prefixed_tool.name = prefixed;
                        tools.push(prefixed_tool);
                    }
                }
                Err(e) => tracing::warn!(error = %e, upstream = idx, "list_tools failed"),
            }
        }

        Some(ListToolsResult {
            tools,
            next_cursor: None,
            _meta: None,
        })
    }

    /// Fan `resources/list` out to every upstream in the session in
    /// parallel, concatenate the per-upstream resource lists, populate
    /// the in-session resource→connection routing map, return the union.
    /// Same best-effort semantics as [`list_tools`](Self::list_tools).
    pub async fn list_resources(&self, session_id: &str) -> Option<ListResourcesResult> {
        let session = self.get(session_id)?;
        let results = join_all(
            session
                .connections
                .iter()
                .map(|c| async move { c.list_resources().await }),
        )
        .await;

        let mut resources: Vec<Resource> = Vec::new();
        for (idx, result) in results.into_iter().enumerate() {
            let connection = &session.connections[idx];
            match result {
                Ok(arc) => {
                    for resource in arc.iter() {
                        let prefixed = prefix_name(connection, &resource.uri);
                        if let Some(prev) = session
                            .resource_owner
                            .insert(prefixed.clone(), (idx, resource.uri.clone()))
                        {
                            if prev.0 != idx {
                                tracing::warn!(
                                    uri = %prefixed,
                                    previous_upstream = prev.0,
                                    new_upstream = idx,
                                    "prefixed resource URI collision; new upstream wins",
                                );
                            }
                        }
                        let mut prefixed_resource = resource.clone();
                        prefixed_resource.uri = prefixed;
                        resources.push(prefixed_resource);
                    }
                }
                Err(e) => tracing::warn!(error = %e, upstream = idx, "list_resources failed"),
            }
        }

        Some(ListResourcesResult {
            resources,
            next_cursor: None,
            _meta: None,
        })
    }

    /// Forward `tools/call` to whichever upstream owns the named tool.
    ///
    /// Routing: look up `params.name` in the cached `tool_owner` map. On
    /// cache miss (e.g. the client called `tools/call` without first
    /// listing), refresh the map by re-fetching every upstream's tool list
    /// in parallel and retry the lookup.
    pub async fn call_tool(
        &self,
        session_id: &str,
        params: &CallToolRequestParams,
    ) -> Result<CallToolResult, CallToolError> {
        let session = self
            .get(session_id)
            .ok_or_else(|| CallToolError::SessionNotFound(session_id.to_string()))?;

        let (idx, original_name) =
            match session.tool_owner.get(&params.name).map(|e| e.value().clone()) {
                Some(v) => v,
                None => {
                    self.refresh_tool_owner(&session).await;
                    session
                        .tool_owner
                        .get(&params.name)
                        .map(|e| e.value().clone())
                        .ok_or_else(|| CallToolError::ToolNotFound(params.name.clone()))?
                }
            };

        let connection = session
            .connections
            .get(idx)
            .ok_or_else(|| CallToolError::ToolNotFound(params.name.clone()))?;
        // Forward to the upstream with the un-prefixed tool name it actually
        // knows; pass everything else (`arguments`, `task`, `_meta`)
        // through unchanged.
        let upstream_params = CallToolRequestParams {
            name: original_name,
            arguments: params.arguments.clone(),
            task: params.task.clone(),
            _meta: params._meta.clone(),
        };
        Ok(connection.call_tool(&upstream_params).await?)
    }

    /// Forward `resources/read` to whichever upstream owns the URI. Same
    /// cache-miss-fallback pattern as [`call_tool`](Self::call_tool).
    pub async fn read_resource(
        &self,
        session_id: &str,
        uri: &str,
    ) -> Result<ReadResourceResult, ReadResourceError> {
        let session = self
            .get(session_id)
            .ok_or_else(|| ReadResourceError::SessionNotFound(session_id.to_string()))?;

        let (idx, original_uri) =
            match session.resource_owner.get(uri).map(|e| e.value().clone()) {
                Some(v) => v,
                None => {
                    self.refresh_resource_owner(&session).await;
                    session
                        .resource_owner
                        .get(uri)
                        .map(|e| e.value().clone())
                        .ok_or_else(|| ReadResourceError::ResourceNotFound(uri.to_string()))?
                }
            };

        let connection = session
            .connections
            .get(idx)
            .ok_or_else(|| ReadResourceError::ResourceNotFound(uri.to_string()))?;
        Ok(connection.read_resource(&original_uri).await?)
    }

    /// Re-fetch every upstream's tool list and rebuild `tool_owner`.
    /// Called as a fallback when [`call_tool`](Self::call_tool) doesn't
    /// find the tool in the cache.
    async fn refresh_tool_owner(&self, session: &Session) {
        let results = join_all(
            session
                .connections
                .iter()
                .map(|c| async move { c.list_tools().await }),
        )
        .await;

        for (idx, result) in results.into_iter().enumerate() {
            let connection = &session.connections[idx];
            match result {
                Ok(arc) => {
                    for tool in arc.iter() {
                        let prefixed = prefix_name(connection, &tool.name);
                        session
                            .tool_owner
                            .insert(prefixed, (idx, tool.name.clone()));
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, upstream = idx, "refresh tool_owner: list_tools failed")
                }
            }
        }
    }

    /// Re-fetch every upstream's resource list and rebuild
    /// `resource_owner`. Called as a fallback when
    /// [`read_resource`](Self::read_resource) doesn't find the URI in the
    /// cache.
    async fn refresh_resource_owner(&self, session: &Session) {
        let results = join_all(
            session
                .connections
                .iter()
                .map(|c| async move { c.list_resources().await }),
        )
        .await;

        for (idx, result) in results.into_iter().enumerate() {
            let connection = &session.connections[idx];
            match result {
                Ok(arc) => {
                    for resource in arc.iter() {
                        let prefixed = prefix_name(connection, &resource.uri);
                        session
                            .resource_owner
                            .insert(prefixed, (idx, resource.uri.clone()));
                    }
                }
                Err(e) => {
                    tracing::warn!(error = %e, upstream = idx, "refresh resource_owner: list_resources failed")
                }
            }
        }
    }
}
