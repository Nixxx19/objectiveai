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

use axum::extract::FromRequestParts;
use axum::extract::ws::{CloseCode, CloseFrame, Message, WebSocket, close_code};
use axum::http::request::Parts;
use axum::response::Response;
use futures::{Stream, StreamExt};
use objectiveai_sdk::error::ResponseError;
use serde::Serialize;

use crate::error::ResponseErrorExt;

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

/// Drain `stream` to `socket`, one JSON text frame per chunk, then
/// `Close(1000)`. If `socket.send` fails (peer hung up), the loop
/// breaks early and no close is attempted.
pub async fn serve_chunks<C, S>(socket: &mut WebSocket, stream: S)
where
    C: Serialize,
    S: Stream<Item = C> + Unpin,
{
    let mut stream = stream;
    while let Some(chunk) = stream.next().await {
        let json = match serde_json::to_string(&chunk) {
            Ok(s) => s,
            Err(_) => continue, // chunk types are infallible to serialize in practice
        };
        if socket.send(Message::Text(json.into())).await.is_err() {
            return;
        }
    }
    let _ = socket
        .send(Message::Close(Some(CloseFrame {
            code: close_code::NORMAL,
            reason: "".into(),
        })))
        .await;
}

/// Drain `stream` (where each item is itself a `Result`) to `socket`.
/// Both `Ok` and `Err` arms are serialized in-band as text frames —
/// `Err` items do **not** terminate the stream. Mirrors the SSE
/// behavior in `create_error` and `create_profile_computation` where
/// per-chunk errors arrive as normal `data:` events alongside `Ok`
/// chunks. Closes `1000` at end of stream.
pub async fn serve_result_chunks<C, E, S>(socket: &mut WebSocket, stream: S)
where
    C: Serialize,
    E: Serialize,
    S: Stream<Item = Result<C, E>> + Unpin,
{
    let mut stream = stream;
    while let Some(item) = stream.next().await {
        let json = match item {
            Ok(chunk) => serde_json::to_string(&chunk),
            Err(err) => serde_json::to_string(&err),
        };
        let json = match json {
            Ok(s) => s,
            Err(_) => continue,
        };
        if socket.send(Message::Text(json.into())).await.is_err() {
            return;
        }
    }
    let _ = socket
        .send(Message::Close(Some(CloseFrame {
            code: close_code::NORMAL,
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
