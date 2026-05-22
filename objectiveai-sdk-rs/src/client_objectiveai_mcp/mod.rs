//! Client-side ObjectiveAI MCP request envelopes.
//!
//! Two enums for the two surfaces of the local objectiveai-mcp:
//!
//! - [`client_request::Request`] — addressed to the client-app layer
//!   (e.g. notify a running agent completion).
//! - [`server_request::Request`] — addressed to the MCP-server layer
//!   (standard MCP `tools/list`, `tools/call`).
//!
//! Sent down the reverse-attach channel from #193 (stage 2+).

pub mod client_request;
pub mod server_request;
