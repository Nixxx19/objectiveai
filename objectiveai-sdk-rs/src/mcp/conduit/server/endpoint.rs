//! Axum handlers for the API's `/objectiveai-mcp/{session_id}` route.
//!
//! Two halves:
//!
//! - [`handle_post_or_delete`] — POST/DELETE/PUT/PATCH bridge. The
//!   inbound HTTP request the proxy made is wrapped as a
//!   `server_request::Request` and forwarded over the matching WS;
//!   the calling client's `McpHandler` (typically
//!   [`super::super::client::Conduit`]) replies with a
//!   `server_response::Response` and we translate it back into the
//!   HTTP response.
//!
//! - [`handle_get_sse`] — GET (Streamable HTTP MCP notifications
//!   stream). Subscribes to the per-`(ws_session_id, mcp_session_id)`
//!   broadcast and emits standard MCP `notifications/<kind>/list_changed`
//!   JSON-RPC envelopes whenever the CLI pushes one up over its
//!   `client_request::Payload::McpListChanged`.

use super::listeners::McpListenerRegistry;
use super::registry::ReverseChannelRegistry;
use super::send::send_server_request;
use crate::client_objectiveai_mcp::client_request::McpListChangedKind;
use crate::client_objectiveai_mcp::server_request;
use axum::body::Bytes;
use axum::http::{HeaderMap, HeaderValue, Method, StatusCode};
use axum::response::{
    IntoResponse, Response,
    sse::{Event, KeepAlive, Sse},
};
use futures::stream::StreamExt;
use indexmap::IndexMap;
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::wrappers::BroadcastStream;

/// How long to wait for a `server_response` before failing with
/// `504 Gateway Timeout`. Matches the proxy's default call timeout.
const REVERSE_CHANNEL_TIMEOUT: Duration = Duration::from_secs(30);

/// SSE keepalive cadence for the GET notifications stream.
const SSE_KEEP_ALIVE: Duration = Duration::from_secs(15);

/// Forward one HTTP request the proxy made to the calling client's
/// reverse-attach handler and translate the reply back into an HTTP
/// response. Mounted as POST/DELETE/PUT/PATCH on the same route — all
/// converge here.
pub async fn handle_post_or_delete(
    session_id: String,
    method: Method,
    registry: ReverseChannelRegistry,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let rc = match registry.get(&session_id) {
        Some(rc) => rc.clone(),
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                format!("no reverse channel for session_id {session_id:?}"),
            )
                .into_response();
        }
    };

    let forward_headers: IndexMap<String, String> = headers
        .iter()
        .filter_map(|(k, v)| Some((k.as_str().to_string(), v.to_str().ok()?.to_string())))
        .collect();

    let body_value: Option<serde_json::Value> = if body.is_empty() {
        None
    } else {
        match serde_json::from_slice(&body) {
            Ok(v) => Some(v),
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    format!("body is not valid JSON: {e}"),
                )
                    .into_response();
            }
        }
    };

    let request_id = uuid::Uuid::new_v4().to_string();
    let request = server_request::Request {
        id: request_id,
        method: method.as_str().to_string(),
        headers: forward_headers,
        body: body_value,
    };

    let rx = match send_server_request(&rc.sink, &rc.pending, request).await {
        Ok(rx) => rx,
        Err(()) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "reverse channel closed before request could be sent",
            )
                .into_response();
        }
    };

    let server_resp = match tokio::time::timeout(REVERSE_CHANNEL_TIMEOUT, rx).await {
        Ok(Ok(r)) => r,
        Ok(Err(_)) => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                "reverse channel dropped before response arrived",
            )
                .into_response();
        }
        Err(_) => {
            return (
                StatusCode::GATEWAY_TIMEOUT,
                "reverse channel timed out waiting for response",
            )
                .into_response();
        }
    };

    let status = StatusCode::from_u16(server_resp.status)
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let mut builder = axum::response::Response::builder().status(status);
    let mut has_content_type = false;
    for (k, v) in &server_resp.headers {
        if let Ok(value) = HeaderValue::from_str(v) {
            if k.eq_ignore_ascii_case("content-type") {
                has_content_type = true;
            }
            builder = builder.header(k, value);
        }
    }
    if !has_content_type && server_resp.body.is_some() {
        builder = builder.header("Content-Type", "application/json");
    }

    let body_bytes: Vec<u8> = match server_resp.body {
        Some(v) => serde_json::to_vec(&v).unwrap_or_default(),
        None => Vec::new(),
    };

    builder
        .body(axum::body::Body::from(body_bytes))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

/// GET `/objectiveai-mcp/{session_id}`: open the per-MCP-session SSE
/// notifications stream the proxy subscribes to for
/// `notifications/{tools,resources}/list_changed`. Requires an
/// `Mcp-Session-Id` request header to identify which upstream MCP
/// connection's events to forward; without one we 400.
///
/// The stream emits standard MCP-spec JSON-RPC envelopes as `data:`
/// frames:
///
/// ```text
/// data: {"jsonrpc":"2.0","method":"notifications/tools/list_changed"}
/// ```
///
/// `KeepAlive` pings every [`SSE_KEEP_ALIVE`] hold the stream open
/// during quiet periods. When the last receiver hangs up the
/// stream's drop guard calls [`McpListenerRegistry::gc`].
pub async fn handle_get_sse(
    session_id: String,
    listeners: McpListenerRegistry,
    headers: HeaderMap,
) -> Response {
    let mcp_session_id = match headers
        .get("Mcp-Session-Id")
        .and_then(|v| v.to_str().ok())
    {
        Some(s) => s.to_string(),
        None => {
            return (
                StatusCode::BAD_REQUEST,
                "Mcp-Session-Id header is required on GET /objectiveai-mcp/{session_id}",
            )
                .into_response();
        }
    };

    let rx = listeners.subscribe(&session_id, &mcp_session_id);

    // Wrap in a drop-guard so the registry GC fires when the
    // subscriber hangs up. `BroadcastStream` itself drops the
    // receiver when iteration stops, but it doesn't know about our
    // registry — we do the call here.
    struct GcGuard {
        listeners: McpListenerRegistry,
        ws_session_id: String,
        mcp_session_id: String,
    }
    impl Drop for GcGuard {
        fn drop(&mut self) {
            self.listeners.gc(&self.ws_session_id, &self.mcp_session_id);
        }
    }
    let gc = GcGuard {
        listeners,
        ws_session_id: session_id,
        mcp_session_id,
    };

    let stream = BroadcastStream::new(rx).filter_map(move |item: Result<
        McpListChangedKind,
        tokio_stream::wrappers::errors::BroadcastStreamRecvError,
    >| {
        // Keep the gc guard alive for the entire stream lifetime by
        // closing over it. The closure is owned by `filter_map`, which
        // is owned by the SSE stream, which is owned by the response;
        // it drops when the client disconnects.
        let _ = &gc;
        async move {
            let kind = item.ok()?;
            let value = serde_json::json!({
                "jsonrpc": "2.0",
                "method": kind.method(),
            });
            let json = serde_json::to_string(&value).ok()?;
            Some(Ok::<_, Infallible>(Event::default().data(json)))
        }
    });

    Sse::new(stream)
        .keep_alive(KeepAlive::new().interval(SSE_KEEP_ALIVE))
        .into_response()
}

