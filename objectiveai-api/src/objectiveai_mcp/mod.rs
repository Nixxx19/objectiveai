//! ObjectiveAI MCP server — Streamable HTTP MCP, mounted under
//! `/objectiveai-mcp` with three routes (POST / GET / DELETE) and
//! routed by the `X-OBJECTIVEAI-RESPONSE-ID` header. Route surface
//! mirrors the MCP-spec subset of `objectiveai-mcp-proxy/src/mcp.rs`;
//! the proxy's ObjectiveAI-specific `/notify` extensions are not
//! mirrored.
//!
//! Each JSON-RPC method delegates to a typed function in [`handlers`]
//! that re-wraps the call as a `server_request::Request` and ships it
//! over the matching reverse-channel WS (see
//! `objectiveai_sdk::mcp::conduit::server`). The CLI conduit on the
//! other side fans out to per-`Mcp-Session-Id` upstream MCP
//! connections.

mod context;
mod handlers;
mod routes;

pub use context::McpRequestContext;
pub use routes::router;
