//! Responses to [`super::server_request::Request`]s — `ok` carries
//! the MCP-shape JSON the client's local objectiveai-mcp produced
//! (e.g. `ListToolsResult`, `CallToolResult`); `error` carries a
//! code + message for failure. The `id` echoes the request's `id`
//! so the API can correlate replies to in-flight requests.

mod response;
pub use response::*;
mod result;
pub use result::*;
