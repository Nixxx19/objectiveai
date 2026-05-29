//! ObjectiveAI MCP server — Streamable HTTP MCP + the `/notify`
//! extensions the agent client uses to inject inter-tool-call
//! `<system-reminder>` content.
//!
//! Route surface mirrors `objectiveai-mcp-proxy/src/mcp.rs`. Each
//! method delegates to a typed function in [`handlers`] whose body
//! is currently `todo!()` — the dispatch (envelope parse, params
//! `from_value`, response framing) is real and exercisable; the
//! handler bodies panic when reached.

mod context;
mod handlers;
mod routes;

pub use context::McpRequestContext;
pub use routes::router;
