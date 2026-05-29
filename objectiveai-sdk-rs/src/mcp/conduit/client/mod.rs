//! Client-app side of the conduit. The CLI hosts this — implements
//! [`crate::http::McpHandler`] to forward inbound `server_request`
//! frames to a real upstream MCP server, cached one
//! [`super::super::Connection`] per remote-minted `Mcp-Session-Id`.

mod handler;

pub use handler::Conduit;
