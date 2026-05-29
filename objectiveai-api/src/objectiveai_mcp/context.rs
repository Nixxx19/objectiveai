//! Shared per-request context handed to every MCP delegate.

use axum::http::HeaderMap;
use objectiveai_sdk::mcp::conduit::server::{
    McpListenerRegistry, ReverseChannelRegistry,
};

/// Shared state every MCP delegate receives. Built once per HTTP
/// request by the axum handler before calling the delegate; cheap
/// to construct (every field is `Clone` of an `Arc`-backed handle).
pub struct McpRequestContext {
    /// URL-path session id — same value the WS reverse-attach
    /// registry was keyed on at upgrade time. Unknown ids are
    /// rejected at the route layer (404) before any delegate sees
    /// this struct.
    pub session_id: String,
    /// Verbatim request headers. Delegates that need to forward
    /// upstream (typically every `tools/*` and `resources/*` method)
    /// read `Mcp-Session-Id` from here.
    pub headers: HeaderMap,
    /// Handle to the per-WS reverse-channel registry — delegates
    /// that forward to the CLI grab the matching sink + pending
    /// registry from this and call
    /// [`objectiveai_sdk::mcp::conduit::server::send_server_request`].
    pub registry: ReverseChannelRegistry,
    /// Shared per-(ws_session_id, mcp_session_id) SSE listener
    /// registry. The SDK's `handle_get_sse` already subscribes here
    /// directly; delegates that inject list-changed events go
    /// through it too.
    pub listeners: McpListenerRegistry,
}
