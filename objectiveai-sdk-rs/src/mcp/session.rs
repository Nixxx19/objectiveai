//! MCP session-id constants shared across SDK / cli / cli-stream / mcp.
//!
//! Forms in this module are kept in sync by hand:
//!
//! - [`MCP_SESSION_ID_HEADER`] — wire HTTP header name (the MCP
//!   Streamable-HTTP standard spelling, mixed-case dashes).
//! - [`MCP_SESSION_ID_ENV`] — environment variable form, used to
//!   propagate the session id into a cli subprocess (e.g. a tool the
//!   cli spawns) without re-encoding it as a header.

/// HTTP header name carrying the MCP session id on requests between
/// MCP clients and servers. Matches the casing used by the upstream
/// rmcp transport.
pub const MCP_SESSION_ID_HEADER: &str = "Mcp-Session-Id";

/// Environment variable spelling of the same id. Used to forward the
/// session id from the cli into tool subprocesses it spawns.
pub const MCP_SESSION_ID_ENV: &str = "MCP_SESSION_ID";
