//! Requests addressed to the MCP-server protocol layer of the local
//! objectiveai-mcp — standard MCP `tools/list` and `tools/call`
//! shapes the API forwards on behalf of its upstream MCP clients.

mod request;
pub use request::*;
