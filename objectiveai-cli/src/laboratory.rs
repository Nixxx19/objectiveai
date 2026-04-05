/// Pre-built `objectiveai-mcp` binary (linux-musl, filesystem feature only).
/// Same architecture as the CLI build target.
pub const MCP_BINARY: &[u8] = include_bytes!(env!("OBJECTIVEAI_MCP_BINARY_PATH"));
