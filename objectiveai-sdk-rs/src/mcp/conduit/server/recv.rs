//! Conduit recv loop. Drains the WS recv-half, classifies each text
//! frame, and dispatches.

use super::registry::{PendingRequests, SessionTracker, SharedSink};
use crate::client_objectiveai_mcp::{
    client_request::{McpListChanged, Payload as ClientPayload, Request as ClientRequest},
    client_response::Response as ClientResponse,
    server_response::Response as ServerResponse,
};
use crate::error::ResponseError;
use axum::extract::ws::{Message, WebSocket};
use futures::SinkExt;
use futures::StreamExt;
use futures::stream::SplitStream;
use std::sync::Arc;

/// Drain the WS recv-half, parse each text frame, dispatch by shape.
///
/// - `client_request::Request` with
///   [`ClientPayload::AgentCompletionNotify`]: validate `response_id`
///   against `tracker`, spawn `notify_fn`, write back a
///   `client_response::Response` echoing the request `id`.
/// - `client_request::Request` with [`ClientPayload::McpListChanged`]:
///   spawn `list_changed_fn`, ack `Ok { id }` immediately (the CLI
///   has no remediation path for fan-out failure).
/// - `server_response::Response`: look up `pending[id]`, fulfill the
///   matching oneshot.
/// - Unknown shape: log + drop.
///
/// Returns when the recv half closes (peer hung up or sent a close
/// frame). Per-message dispatches are spawned so the loop keeps
/// servicing `server_response` correlations even while a slow
/// `notify_fn` is in flight.
pub async fn recv_loop<NF, NFut, LCF, LCFut>(
    mut rx: SplitStream<WebSocket>,
    tracker: Arc<SessionTracker>,
    sink: SharedSink,
    pending: PendingRequests,
    notify_fn: NF,
    list_changed_fn: LCF,
) where
    NF: Fn(crate::agent::completions::request::AgentCompletionNotifyParams) -> NFut
        + Send
        + Sync
        + 'static,
    NFut: std::future::Future<Output = Result<(), ResponseError>> + Send + 'static,
    LCF: Fn(McpListChanged) -> LCFut + Send + Sync + 'static,
    LCFut: std::future::Future<Output = ()> + Send + 'static,
{
    // Arc-wrap so each spawned dispatch task can hold its own cheap
    // clone without forcing the caller's closures to be Arc-typed.
    let notify_fn = Arc::new(notify_fn);
    let list_changed_fn = Arc::new(list_changed_fn);

    loop {
        let msg = match rx.next().await {
            Some(m) => m,
            None => return,
        };
        let text = match msg {
            Ok(Message::Text(t)) => t,
            Ok(Message::Binary(_)) => continue,
            Ok(Message::Ping(_) | Message::Pong(_)) => continue,
            Ok(Message::Close(_)) => return,
            Err(_) => return,
        };

        // Parse strategy: try client_request first (the discriminator
        // tag `type` distinguishes it from server_response — they
        // share the `id` field but differ everywhere else), then
        // server_response, then drop.
        if let Ok(request) = serde_json::from_str::<ClientRequest>(text.as_str()) {
            let ClientRequest { id, payload } = request;
            match payload {
                ClientPayload::AgentCompletionNotify(params) => {
                    let tracker = tracker.clone();
                    let sink = sink.clone();
                    let notify_fn = notify_fn.clone();
                    tokio::spawn(async move {
                        let response: ClientResponse = if !tracker.contains(&params.response_id) {
                            ClientResponse::Error {
                                id,
                                code: 404,
                                message: serde_json::Value::String(format!(
                                    "response_id {:?} not from this stream",
                                    params.response_id
                                )),
                            }
                        } else {
                            match (notify_fn)(params).await {
                                Ok(()) => ClientResponse::Ok { id },
                                Err(e) => ClientResponse::Error {
                                    id,
                                    code: e.code,
                                    message: e.message,
                                },
                            }
                        };
                        write_client_response(&sink, &response).await;
                    });
                    continue;
                }
                ClientPayload::McpListChanged(change) => {
                    let sink = sink.clone();
                    let list_changed_fn = list_changed_fn.clone();
                    tokio::spawn(async move {
                        (list_changed_fn)(change).await;
                        write_client_response(&sink, &ClientResponse::Ok { id }).await;
                    });
                    continue;
                }
            }
        }

        if let Ok(response) = serde_json::from_str::<ServerResponse>(text.as_str()) {
            if let Some((_, tx)) = pending.remove(&response.id) {
                let _ = tx.send(response);
            }
            continue;
        }
        // Unknown frame shape — drop silently. We previously logged
        // these but the noise outweighed the diagnostic value.
    }
}

async fn write_client_response(sink: &SharedSink, response: &ClientResponse) {
    let frame = match serde_json::to_string(response) {
        Ok(s) => s,
        Err(_) => return,
    };
    let mut guard = sink.lock().await;
    let _ = guard.send(Message::Text(frame.into())).await;
}
