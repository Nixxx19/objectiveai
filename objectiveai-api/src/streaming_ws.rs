//! Transport selection + WebSocket transport helpers for the streaming endpoints.
//!
//! Each streaming endpoint (`/agent/completions`, `/vector/completions`,
//! etc.) lives behind a single `axum::routing::any(...)` route. The
//! handler inspects the `X-Transport` request header via the
//! [`Transport`] extractor and forks:
//!
//! - `X-Transport: sse` → POST + JSON body, response is `text/event-stream`
//!   (the existing SSE handler).
//! - Anything else (including missing header) → GET + `Upgrade: websocket`,
//!   response is a WebSocket text-frame stream (the `_ws` handler).
//!
//! WS wire protocol after the upgrade:
//!
//! - Client → server: one text frame with the JSON request body
//!   (`*CreateParams`), exactly the same shape the SSE branch
//!   deserializes from the POST body.
//! - Server → client: N text frames, one chunk per frame, JSON
//!   encoded — same `*Chunk` types each endpoint already emits.
//! - End of stream: server sends `Close(1000)`. No `[DONE]` sentinel.
//! - Error mid-stream: server sends one final text frame containing
//!   the JSON `ResponseError`, then `Close(1011)`.
//! - Body parse failure: error text frame, `Close(1003)`.
//!
//! Auth lives on the upgrade handshake (`Authorization` header), the
//! same place every other route validates it; the helpers below are
//! invoked only after the upgrade has been accepted.
//!
//! Stage 1 of #193; #194 tracks the migration.

use std::sync::Arc;

use axum::extract::FromRequestParts;
use axum::extract::ws::{CloseCode, CloseFrame, Message, WebSocket, close_code};
use axum::http::request::Parts;
use axum::response::Response;
use futures::{SinkExt, StreamExt};
use futures::stream::{SplitSink, SplitStream};
use objectiveai_sdk::error::ResponseError;
use serde::Serialize;
use tokio::sync::Mutex;

use crate::error::ResponseErrorExt;

/// Shared sender half of a split WebSocket, wrapped under a tokio
/// mutex so the send-side (chunk forwarder) and recv-side (notify
/// responder) can both write frames safely. Locks are short-lived —
/// only held across a single `send`.
pub type SharedSink = Arc<Mutex<SplitSink<WebSocket, Message>>>;

/// Per-WS-connection tracker of agent-completion `response_id`s
/// emitted by this stream. Populated on the send side as each chunk
/// flows out (via [`AgentCompletionIds`]) and read on the recv side
/// to validate incoming notify requests' `response_id`. Notifies
/// targeting an id not in this tracker are rejected with 404.
pub struct SessionTracker {
    ids: dashmap::DashSet<String>,
}

impl SessionTracker {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            ids: dashmap::DashSet::new(),
        })
    }

    /// Extend the tracker with every agent-completion id this chunk
    /// carries. Borrows into the chunk; no allocation beyond the
    /// `insert` itself.
    pub fn observe<C>(&self, chunk: &C)
    where
        C: objectiveai_sdk::agent::completions::response::streaming::AgentCompletionIds,
    {
        for id in chunk.agent_completion_ids() {
            self.ids.insert(id.to_string());
        }
    }

    pub fn contains(&self, id: &str) -> bool {
        self.ids.contains(id)
    }
}

/// Transport the client wants. Set via the `X-Transport` request
/// header; missing or unknown values default to [`Transport::WebSocket`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transport {
    Sse,
    WebSocket,
}

impl<S> FromRequestParts<S> for Transport
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;
    async fn from_request_parts(parts: &mut Parts, _: &S) -> Result<Self, Self::Rejection> {
        let header = parts
            .headers
            .get("X-Transport")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        Ok(if header.eq_ignore_ascii_case("sse") {
            Transport::Sse
        } else {
            Transport::WebSocket
        })
    }
}

/// Render a `400 Bad Request` response with the given message in a
/// JSON `ResponseError` envelope. Used by the transport dispatcher
/// when the client's combination of method + headers + body doesn't
/// match the transport they asked for.
pub fn bad_request(message: &str) -> Response {
    ResponseError {
        code: 400,
        message: serde_json::Value::String(message.to_string()),
    }
    .into_response()
}
use serde::de::DeserializeOwned;

/// Read exactly one text frame from `socket` and deserialize it as `T`.
///
/// Skips pings/pongs/binary frames silently — only a text frame is a
/// valid body. Returns a `ResponseError` describing the failure if
/// the peer closes early, sends something we can't parse, or sends a
/// non-text frame.
///
/// Caller is responsible for closing the socket on error (typically
/// via [`send_error_and_close`]).
pub async fn recv_body_frame<T: DeserializeOwned>(
    socket: &mut WebSocket,
) -> Result<T, ResponseError> {
    loop {
        match socket.recv().await {
            Some(Ok(Message::Text(text))) => {
                return serde_json::from_str::<T>(text.as_str()).map_err(|e| ResponseError {
                    code: 400,
                    message: serde_json::Value::String(format!(
                        "failed to deserialize body frame: {e}"
                    )),
                });
            }
            Some(Ok(Message::Binary(_))) => {
                return Err(ResponseError {
                    code: 400,
                    message: serde_json::Value::String(
                        "expected text body frame, got binary".into(),
                    ),
                });
            }
            // Library handles ping/pong automatically; ignore if surfaced.
            Some(Ok(Message::Ping(_) | Message::Pong(_))) => continue,
            Some(Ok(Message::Close(_))) | None => {
                return Err(ResponseError {
                    code: 400,
                    message: serde_json::Value::String(
                        "peer closed before sending body".into(),
                    ),
                });
            }
            Some(Err(e)) => {
                return Err(ResponseError {
                    code: 400,
                    message: serde_json::Value::String(format!("websocket recv error: {e}")),
                });
            }
        }
    }
}

/// Send `err` as a single text frame, then close with `code`.
///
/// Failures to send are swallowed — the socket is being torn down
/// anyway, and the peer can only do one of the two no-ops (notice the
/// close, or notice nothing because they've already gone).
pub async fn send_error_and_close(socket: &mut WebSocket, err: &ResponseError, code: CloseCode) {
    let frame = serde_json::to_string(err).unwrap_or_else(|_| String::from("{}"));
    let _ = socket.send(Message::Text(frame.into())).await;
    let _ = socket
        .send(Message::Close(Some(CloseFrame {
            code,
            reason: "".into(),
        })))
        .await;
}

/// Close the socket with `Close(1011)` after sending the given
/// `ResponseError` as a text frame. Used when setup (e.g.
/// `create_streaming_handle_usage`) fails before any chunk has been
/// produced.
pub async fn fatal_setup_error(socket: &mut WebSocket, err: &ResponseError) {
    send_error_and_close(socket, err, close_code::ERROR).await;
}

// ────────────────────────────────────────────────────────────────────
// Split-sink variants. Used by `_ws` handlers after splitting the
// socket so the send-side (chunk forwarder) and recv-side (notify
// responder) can write through the same socket concurrently.
// ────────────────────────────────────────────────────────────────────

/// Send one chunk as a text frame. Caller observes the chunk into the
/// session tracker beforehand. Returns `Err(())` if the peer hung up.
pub async fn send_chunk_split<C: Serialize>(sink: &SharedSink, chunk: &C) -> Result<(), ()> {
    let json = match serde_json::to_string(chunk) {
        Ok(s) => s,
        Err(_) => return Ok(()), // chunk types are infallible to serialize in practice
    };
    let mut guard = sink.lock().await;
    guard
        .send(Message::Text(json.into()))
        .await
        .map_err(|_| ())
}

/// Send a `Close(code)` frame, ignoring any I/O error.
pub async fn send_close_split(sink: &SharedSink, code: CloseCode) {
    let mut guard = sink.lock().await;
    let _ = guard
        .send(Message::Close(Some(CloseFrame {
            code,
            reason: "".into(),
        })))
        .await;
}

/// Recv loop: drain the split stream, parse each text frame as
/// [`client_request::Request`](objectiveai_sdk::client_objectiveai_mcp::client_request::Request),
/// validate the `response_id` against the session tracker, and
/// dispatch to `notify_fn` on hit. Writes the matching
/// [`client_response::Response`](objectiveai_sdk::client_objectiveai_mcp::client_response::Response)
/// back through `sink` for every parsed request (success → `Ok`,
/// failure → `Error { code, message }`).
///
/// Frames that don't parse as a `Request` are logged at `warn!` and
/// dropped — no response is emitted because there's no `id` to
/// correlate to.
///
/// Returns when the recv half closes (peer hung up or close frame).
pub async fn recv_notify_loop<F, Fut>(
    mut rx: SplitStream<WebSocket>,
    tracker: Arc<SessionTracker>,
    sink: SharedSink,
    notify_fn: F,
) where
    F: Fn(objectiveai_sdk::agent::completions::request::AgentCompletionNotifyParams) -> Fut
        + Send
        + Sync,
    Fut: std::future::Future<Output = Result<(), crate::agent::completions::Error>> + Send,
{
    use objectiveai_sdk::client_objectiveai_mcp::{
        client_request::{Payload, Request},
        client_response::{Response as ClientResponse, Result as ClientResult},
    };

    while let Some(msg) = rx.next().await {
        let text = match msg {
            Ok(Message::Text(t)) => t,
            Ok(Message::Binary(_)) => {
                eprintln!("ignoring binary frame on streaming WS recv side");
                continue;
            }
            Ok(Message::Ping(_) | Message::Pong(_)) => continue,
            Ok(Message::Close(_)) => return,
            Err(e) => {
                eprintln!("streaming WS recv error: {e}");
                return;
            }
        };

        let request: Request = match serde_json::from_str(text.as_str()) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("dropping unparseable client_request frame: {e}");
                continue;
            }
        };

        let Request { id, payload } = request;
        match payload {
            Payload::AgentCompletionNotify(params) => {
                let result: ClientResult = if !tracker.contains(&params.response_id) {
                    ClientResult::Error {
                        code: 404,
                        message: serde_json::Value::String(format!(
                            "response_id {:?} not from this stream",
                            params.response_id
                        )),
                    }
                } else {
                    match notify_fn(params).await {
                        Ok(()) => ClientResult::Ok,
                        Err(e) => {
                            let inner = ResponseError::from(&e);
                            ClientResult::Error {
                                code: inner.code,
                                message: inner.message,
                            }
                        }
                    }
                };
                let response = ClientResponse { id, result };
                let frame = match serde_json::to_string(&response) {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                let mut guard = sink.lock().await;
                if guard.send(Message::Text(frame.into())).await.is_err() {
                    return;
                }
            }
        }
    }
}
