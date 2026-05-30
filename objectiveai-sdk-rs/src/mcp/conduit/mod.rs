//! `objectiveai-mcp` conduit — bidirectional MCP-over-WebSocket bridge
//! between an `objectiveai-api` server and an `objectiveai-cli-stream`
//! client.
//!
//! Only the **server** half lives here now — the API consumes it for
//! its `/objectiveai-mcp` reverse-attach plumbing, recv loop, SSE
//! list-changed forwarding, and `send_server_request` helper. The
//! client half (the per-WS handler that forwards `server_request`
//! frames to a real upstream MCP server and pumps `list_changed`
//! events back) lives in `objectiveai-cli-stream/src/api/conduit.rs`
//! as `ConduitMcpHandler`; the canonical place for new code on the
//! client side is the CLI itself, not this crate.
//!
//! Eventual end-state: this `server` submodule moves into
//! `objectiveai-api/src/objectiveai_mcp/` too and this whole conduit
//! module is deleted. Tracked separately.
//!
//! Wire shapes (`client_request` / `client_response` / `server_request`
//! / `server_response`) live at [`crate::client_objectiveai_mcp`] —
//! they're un-gated so `http::Notifier` consumers can reach them
//! without pulling in `mcp`. The [`wire`] re-export here is the
//! canonical path for code inside the conduit module tree.

pub mod server;

/// Re-export of the conduit's wire shapes. The types physically live
/// at [`crate::client_objectiveai_mcp`] (un-gated) so non-conduit
/// `http` consumers can use them without depending on `mcp`; this
/// alias is the canonical name for code inside `mcp::conduit`.
pub use crate::client_objectiveai_mcp as wire;
