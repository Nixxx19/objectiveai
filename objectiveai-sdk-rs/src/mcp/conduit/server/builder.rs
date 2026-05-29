//! Handler-style builder API. The API server constructs one
//! [`Conduit`] per route family at startup; each accepted WS upgrade
//! calls [`Conduit::attach`] to splice itself into the conduit's
//! dispatch.
//!
//! Usage from an `_ws` handler:
//!
//! ```ignore
//! let conduit = Conduit::builder()
//!     .registry(state.reverse_channels.clone())
//!     .listeners(state.mcp_listeners.clone())
//!     .session_tracker(SessionTracker::new())
//!     .notify_handler(|params| async move {
//!         agent_client.notify(params).await.map_err(|e| e.into())
//!     })
//!     .build();
//!
//! ws.on_upgrade(move |socket| async move {
//!     let attached = conduit.attach(socket);
//!     let send = async move { /* chunk forwarder using attached.sink */ };
//!     tokio::select! {
//!         _ = send => {},
//!         _ = attached.recv => {},
//!     }
//!     // drop(attached.guard) deregisters every ws_session_id
//! });
//! ```

use super::listeners::McpListenerRegistry;
use super::recv::recv_loop;
use super::registry::{
    new_pending_requests, PendingRequests, ReverseAttachGuard, ReverseChannelRegistry,
    SessionTracker, SharedSink,
};
use crate::client_objectiveai_mcp::client_request::McpListChanged;
use crate::error::ResponseError;
use axum::extract::ws::WebSocket;
use futures::StreamExt;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Boxed-future shape every `notify_handler` closure normalizes to.
type NotifyFut =
    Pin<Box<dyn Future<Output = Result<(), ResponseError>> + Send + 'static>>;

/// Boxed `Fn(params) -> NotifyFut` so the builder can store it
/// type-erased without smearing generics over `Conduit`.
type NotifyDispatcher = Arc<
    dyn Fn(crate::agent::completions::request::AgentCompletionNotifyParams) -> NotifyFut
        + Send
        + Sync
        + 'static,
>;

/// Per-WS state returned by [`Conduit::attach`].
pub struct Attached {
    /// Calling endpoint's chunk forwarder writes here.
    pub sink: SharedSink,
    /// Used by the API's `/objectiveai-mcp/{session_id}` POST/DELETE
    /// route to park awaits for `server_response` frames it kicked
    /// off via `send_server_request`.
    pub pending: PendingRequests,
    /// RAII — on drop, every `ws_session_id` registered through this
    /// handle's [`guard().handle()`](ReverseAttachGuard::handle) is
    /// removed from the [`ReverseChannelRegistry`].
    pub guard: ReverseAttachGuard,
    /// Recv future. Caller runs in a `tokio::select!` alongside the
    /// chunk-forwarder send loop. Returns when the peer closes or
    /// sends a `Close` frame.
    pub recv: Pin<Box<dyn Future<Output = ()> + Send>>,
}

struct Inner {
    registry: ReverseChannelRegistry,
    listeners: McpListenerRegistry,
    session_tracker: Arc<SessionTracker>,
    notify_handler: NotifyDispatcher,
}

/// Server-side conduit. Cheap to clone — one per API route family at
/// startup. Holds the cross-WS state (registries, dispatchers); each
/// WS gets its own per-attach state via [`Self::attach`].
#[derive(Clone)]
pub struct Conduit {
    inner: Arc<Inner>,
}

/// Fluent builder for [`Conduit`].
pub struct Builder {
    registry: Option<ReverseChannelRegistry>,
    listeners: Option<McpListenerRegistry>,
    session_tracker: Option<Arc<SessionTracker>>,
    notify_handler: Option<NotifyDispatcher>,
}

impl Conduit {
    /// Start a fluent build chain. Every setter is required —
    /// [`Builder::build`] panics if any is omitted.
    pub fn builder() -> Builder {
        Builder {
            registry: None,
            listeners: None,
            session_tracker: None,
            notify_handler: None,
        }
    }

    /// Splice a freshly-upgraded WebSocket into this conduit. Splits
    /// the socket, registers the reverse channel (handle returned via
    /// `Attached.guard.handle()`), and prepares the recv future the
    /// caller awaits in its `tokio::select!`.
    pub fn attach(&self, ws: WebSocket) -> Attached {
        let (tx, rx_stream) = ws.split();
        let sink: SharedSink = Arc::new(Mutex::new(tx));
        let pending = new_pending_requests();
        let guard = ReverseAttachGuard::new(
            self.inner.registry.clone(),
            sink.clone(),
            pending.clone(),
        );
        let handle = guard.handle();

        // Per-WS list_changed dispatcher: when a McpListChanged
        // arrives over this WS, publish to every ws_session_id this
        // handle has registered. The CLI doesn't know the
        // ws_session_id — we resolve it here from the registry's
        // per-WS view.
        let list_changed_fn = {
            let listeners = self.inner.listeners.clone();
            let handle = handle.clone();
            move |m: McpListChanged| {
                let listeners = listeners.clone();
                let handle = handle.clone();
                async move {
                    for ws_session_id in handle.registered_ids() {
                        listeners.publish(&ws_session_id, &m.mcp_session_id, m.kind);
                    }
                }
            }
        };

        // Stamp the per-WS notify dispatcher into the closure shape
        // recv_loop expects.
        let notify_handler = self.inner.notify_handler.clone();
        let notify_fn = move |p| {
            let notify_handler = notify_handler.clone();
            async move { notify_handler(p).await }
        };

        let recv = Box::pin(recv_loop(
            rx_stream,
            self.inner.session_tracker.clone(),
            sink.clone(),
            pending.clone(),
            notify_fn,
            list_changed_fn,
        ));

        Attached {
            sink,
            pending,
            guard,
            recv,
        }
    }
}

impl Builder {
    pub fn registry(mut self, r: ReverseChannelRegistry) -> Self {
        self.registry = Some(r);
        self
    }

    pub fn listeners(mut self, l: McpListenerRegistry) -> Self {
        self.listeners = Some(l);
        self
    }

    pub fn session_tracker(mut self, t: Arc<SessionTracker>) -> Self {
        self.session_tracker = Some(t);
        self
    }

    /// Closure dispatched by the recv loop on every inbound
    /// `client_request::Payload::AgentCompletionNotify`. Return `Ok`
    /// to ack `Ok { id }`; return `Err(ResponseError)` to ack
    /// `Error { id, code, message }`.
    pub fn notify_handler<F, Fut>(mut self, f: F) -> Self
    where
        F: Fn(crate::agent::completions::request::AgentCompletionNotifyParams) -> Fut
            + Send
            + Sync
            + 'static,
        Fut: Future<Output = Result<(), ResponseError>> + Send + 'static,
    {
        self.notify_handler =
            Some(Arc::new(move |p| Box::pin(f(p)) as NotifyFut));
        self
    }

    /// Finalize. Panics if any setter was skipped — every field is
    /// required for the conduit to be usable.
    pub fn build(self) -> Conduit {
        Conduit {
            inner: Arc::new(Inner {
                registry: self
                    .registry
                    .expect("Conduit::builder().registry(...) required"),
                listeners: self
                    .listeners
                    .expect("Conduit::builder().listeners(...) required"),
                session_tracker: self
                    .session_tracker
                    .expect("Conduit::builder().session_tracker(...) required"),
                notify_handler: self
                    .notify_handler
                    .expect("Conduit::builder().notify_handler(...) required"),
            }),
        }
    }
}
