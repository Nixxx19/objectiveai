//! Per-session state and per-session dispatch.
//!
//! A `Session` owns the upstream MCP connections that belong to one MCP
//! session and is responsible for fanning `tools/list` / `resources/list`
//! out to them and routing `tools/call` / `resources/read` to the right
//! upstream. The registry that minds session ids and hands out
//! `Arc<Session>`s lives in [`crate::session_manager`].

use std::ops::Deref;
use std::sync::Arc;

use futures::future::join_all;
use indexmap::IndexMap;
use objectiveai::mcp::{
    Connection,
    resource::{ListResourcesResult, ReadResourceResult, Resource},
    tool::{CallToolRequestParams, CallToolResult, ListToolsResult, Tool},
};
use tokio::sync::Notify;

/// Owned wrapper around an upstream `Arc<Connection>` whose `Drop` fires
/// the connection's `external_dropped` `Notify`.
///
/// The point: the upstream `Connection`'s background SSE listener is
/// parked on `next_line().await` for the upstream's whole keepalive
/// interval (often 15-60 s). When the proxy drops a session it wants the
/// listener to wake **immediately**. The plain `Arc<Connection>` Drop
/// only fires when the strong count hits zero, but the listener itself
/// holds an `Arc<Connection>` for the duration of one read cycle, so
/// "strong count → 0" never happens until the read cycle ends.
///
/// `UpstreamConnection`'s Drop fires `notify_waiters()` on every drop,
/// which the listener `select!`s against. The listener wakes, re-checks
/// `Arc::strong_count`, and exits when it sees `1` (only itself).
///
/// Deref'ing exposes all `Connection` methods transparently.
#[derive(Debug)]
pub struct UpstreamConnection {
    inner: Arc<Connection>,
    notify: Arc<Notify>,
}

impl UpstreamConnection {
    pub fn new(inner: Arc<Connection>) -> Self {
        let notify = Arc::clone(&inner.external_dropped);
        Self { inner, notify }
    }
}

impl Deref for UpstreamConnection {
    type Target = Connection;
    fn deref(&self) -> &Connection {
        &self.inner
    }
}

impl Drop for UpstreamConnection {
    fn drop(&mut self) {
        // Wake the upstream's listener so it can re-check liveness now,
        // not on the next SSE keepalive. Cheap signal: just sets wakers.
        self.notify.notify_waiters();
    }
}

/// Per-session state.
///
/// All routing, fan-out, and forwarding methods live here. The registry
/// that hands out `Arc<Session>`s by id is `SessionManager` (in
/// [`crate::session_manager`]).
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
    pub connections: IndexMap<String, UpstreamConnection>,
}

impl Session {
    pub(crate) fn new(connections: IndexMap<String, UpstreamConnection>) -> Self {
        Self { connections }
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
    fn route<'a>(&'a self, prefixed: &str) -> Option<(&'a UpstreamConnection, String)> {
        let mut best: Option<(&'a str, &'a UpstreamConnection)> = None;
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

