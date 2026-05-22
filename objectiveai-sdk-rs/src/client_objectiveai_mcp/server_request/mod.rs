//! Requests addressed to the MCP-server protocol layer of the local
//! objectiveai-mcp — standard MCP `tools/list` and `tools/call`
//! shapes the API forwards on behalf of its upstream MCP clients.
//!
//! Each request carries a server-minted `id` that the client echoes
//! in the matching [`super::server_response::Response`] so the server
//! can correlate replies to in-flight requests.

mod request;
pub use request::*;
mod payload;
pub use payload::*;
