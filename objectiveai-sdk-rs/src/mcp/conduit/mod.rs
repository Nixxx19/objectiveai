//! `objectiveai-mcp` conduit — bidirectional MCP-over-WebSocket bridge
//! between an `objectiveai-api` server and an `objectiveai-cli`
//! (or `objectiveai-cli-stream`) client.
//!
//! The conduit factors out logic that previously lived duplicated
//! across `objectiveai-cli`, `objectiveai-cli-stream`, and
//! `objectiveai-api`. Two sides:
//!
//! - [`client`] — the **client-app** side (CLI / cli-stream). Hosts
//!   an [`crate::http::McpHandler`] that forwards inbound
//!   `server_request` frames to a real upstream MCP server, caches
//!   one [`super::Connection`] per `Mcp-Session-Id` it observes, and
//!   (once the list-changed forwarding lands) pushes the upstream's
//!   `notifications/{tools,resources}/list_changed` events back up
//!   the WebSocket as `client_request` frames.
//!
//! - `server` (future) — the **api-host** side. Owns the
//!   reverse-attach registration that lets the api forward proxy
//!   traffic over an in-flight WebSocket, receives the CLI's
//!   list-changed push notifications, and fans them out as SSE
//!   events on the API's `/objectiveai-mcp/{session_id}` MCP
//!   endpoint so the agent's `mcp::Connection` sees them.
//!
//! Wire shapes (`client_request` / `client_response` / `server_request`
//! / `server_response`) live at [`crate::client_objectiveai_mcp`] —
//! they're un-gated so `http::Notifier` consumers can reach them
//! without pulling in `mcp`. The [`wire`] re-export here is the
//! canonical path for code inside the conduit module tree.

pub mod client;

/// Re-export of the conduit's wire shapes. The types physically live
/// at [`crate::client_objectiveai_mcp`] (un-gated) so non-conduit
/// `http` consumers can use them without depending on `mcp`; this
/// alias is the canonical name for code inside `mcp::conduit`.
pub use crate::client_objectiveai_mcp as wire;
