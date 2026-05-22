//! Handler trait for inbound objectiveai-mcp `server_request` frames.
//!
//! The API speaks a bidirectional WS protocol on its streaming
//! endpoints: outbound chunks coexist with outbound MCP `tools/list`
//! / `tools/call` requests the proxy forwards from
//! `/objectiveai-mcp/{session_id}`. Clients that want to expose
//! objectiveai-mcp tools to the agent supply an `McpHandler` to
//! [`crate::http::HttpClient::send_streaming_ws`]; the SDK demuxes
//! frames and dispatches each `server_request::Payload` to it,
//! writing the returned `server_response::Result` back as the
//! matching reply.
//!
//! Clients that don't expose objectiveai-mcp use [`RejectHandler`],
//! which errors every request with `code: 501`.

use crate::client_objectiveai_mcp::{server_request, server_response};
use std::future::Future;

/// Handler for inbound `server_request` frames on a streaming WS.
///
/// One handler instance is bound at `create_streaming` time and
/// stays live for the lifetime of the WS session. Implementations
/// must be `Send + Sync + 'static` since the demux task that
/// invokes them is spawned.
pub trait McpHandler: Send + Sync + 'static {
    /// Dispatch a single request. The return future's `Result` is
    /// written back as the matching `server_response::Response` —
    /// `Ok` carries the MCP-shape JSON the proxy expects (e.g.
    /// `ListToolsResult`), `Error` carries a code + message.
    fn handle(
        &self,
        request: server_request::Payload,
    ) -> impl Future<Output = server_response::Result> + Send;
}

/// Default handler that rejects every `server_request` with
/// `code: 501`. Used when the calling client doesn't host the local
/// objectiveai-mcp surface — agents that declare
/// `client_objectiveai_mcp` will see this and fall through to the
/// next fallback agent on the server side.
#[derive(Debug, Clone, Copy, Default)]
pub struct RejectHandler;

impl McpHandler for RejectHandler {
    async fn handle(&self, _request: server_request::Payload) -> server_response::Result {
        server_response::Result::Error {
            code: 501,
            message: serde_json::Value::String(
                "this client does not host objectiveai-mcp".into(),
            ),
        }
    }
}
