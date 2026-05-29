//! Server-side of the MCP conduit — what the API hosts.
//!
//! Three responsibilities, three layers:
//!
//! 1. **Reverse-attach plumbing** (`registry`) — per-WebSocket sink +
//!    pending-request registry, indexed by `ws_session_id`. The
//!    `_ws` handlers in the api stand one up per accepted upgrade;
//!    the `/objectiveai-mcp/{session_id}` HTTP route looks up the
//!    matching channel and forwards each proxy request over it as a
//!    `server_request::Request`.
//!
//! 2. **WS frame dispatch** (`send` + `recv`) — write helpers for
//!    chunks / errors / `server_request` frames, and the demux loop
//!    that routes inbound `client_request` (`AgentCompletionNotify`,
//!    `McpListChanged`) and `server_response` frames to their
//!    handlers.
//!
//! 3. **MCP-spec endpoint** (`endpoint` + `listeners`) — the actual
//!    Streamable HTTP MCP server the proxy talks to. POST/DELETE
//!    forward over the WS; GET opens the per-`(ws_session_id,
//!    mcp_session_id)` SSE notifications stream that the CLI's
//!    `McpListChanged` pushes feed.
//!
//! The [`Conduit`] type in `builder` glues all three together —
//! that's the handler-style entry point the api's `_ws` handlers
//! consume.

mod builder;
mod endpoint;
mod listeners;
mod recv;
mod registry;
mod send;

pub use builder::{Attached, Builder, Conduit};
pub use endpoint::{handle_get_sse, handle_post_or_delete};
pub use listeners::McpListenerRegistry;
pub use recv::recv_loop;
pub use registry::{
    PendingRequests, ReverseAttachConfig, ReverseAttachGuard, ReverseAttachHandle,
    ReverseChannel, ReverseChannelRegistry, SessionTracker, SharedSink,
    new_pending_requests, new_reverse_channel_registry,
};
pub use send::{
    fatal_setup_error, fatal_setup_error_split, recv_body_frame, send_chunk_split,
    send_close_split, send_error_and_close, send_server_request,
};
