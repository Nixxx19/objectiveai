//! Frame-write helpers for the API side of the conduit.
//!
//! Both the chunk forwarder (the `_ws` handler's send branch) and
//! the MCP endpoint (writing `server_request` frames out) call into
//! these. Locks held are short — only across a single `.send()`.

use super::registry::{PendingRequests, SharedSink};
use crate::client_objectiveai_mcp::{server_request, server_response};
use crate::error::ResponseError;
use axum::extract::ws::{CloseCode, CloseFrame, Message, WebSocket, close_code};
use futures::SinkExt;
use serde::Serialize;
use serde::de::DeserializeOwned;
use tokio::sync::oneshot;

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

/// Send `err` as a single text frame, then close with `code`. Failures
/// to send are swallowed — the socket is being torn down anyway.
pub async fn send_error_and_close(
    socket: &mut WebSocket,
    err: &ResponseError,
    code: CloseCode,
) {
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

/// Split-sink variant of [`fatal_setup_error`]. Used after the socket
/// has already been split (which is the order the `_ws` handlers use
/// so the reverse-attach guard can be built before stream creation).
pub async fn fatal_setup_error_split(sink: &SharedSink, err: &ResponseError) {
    let frame = serde_json::to_string(err).unwrap_or_else(|_| String::from("{}"));
    {
        let mut guard = sink.lock().await;
        let _ = guard.send(Message::Text(frame.into())).await;
    }
    send_close_split(sink, close_code::ERROR).await;
}

/// Send one chunk as a text frame. Caller observes the chunk into the
/// session tracker beforehand. Returns `Err(())` if the peer hung up.
pub async fn send_chunk_split<C: Serialize>(
    sink: &SharedSink,
    chunk: &C,
) -> Result<(), ()> {
    let json = match serde_json::to_string(chunk) {
        Ok(s) => s,
        Err(_) => return Ok(()),
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

/// Register a oneshot under `request.id`, write the request as a text
/// frame, and return the receiver. The caller is responsible for
/// minting the id (and putting it on the request) and applying a
/// timeout (via `tokio::time::timeout`) on the await. On connection
/// drop the recv loop returns and pending oneshots are dropped —
/// receivers observe the close as `Err(RecvError)`.
pub async fn send_server_request(
    sink: &SharedSink,
    pending: &PendingRequests,
    request: server_request::Request,
) -> Result<oneshot::Receiver<server_response::Response>, ()> {
    let id = request.id.clone();
    let (tx, rx) = oneshot::channel();
    pending.insert(id.clone(), tx);

    let frame = match serde_json::to_string(&request) {
        Ok(s) => s,
        Err(_) => {
            pending.remove(&id);
            return Err(());
        }
    };
    let mut guard = sink.lock().await;
    let send_result = guard.send(Message::Text(frame.into())).await;
    if send_result.is_err() {
        drop(guard);
        pending.remove(&id);
        return Err(());
    }
    Ok(rx)
}
