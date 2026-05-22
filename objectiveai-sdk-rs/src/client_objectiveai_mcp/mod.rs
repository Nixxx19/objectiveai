//! Client-side ObjectiveAI MCP envelopes — the shapes the API pushes
//! down the reverse-attach channel from #193 (stage 2+).
//!
//! Three surfaces:
//!
//! - [`client_request::Request`] — addressed to the client-app layer
//!   (e.g. notify a running agent completion). Carries a correlation `id`.
//! - [`client_response::Response`] — server's reply to a
//!   `client_request::Request`. Echoes the request's `id`.
//! - [`server_request::Request`] — addressed to the MCP-server layer
//!   (standard MCP `tools/list`, `tools/call`).

pub mod client_request;
pub mod client_response;
pub mod server_request;
