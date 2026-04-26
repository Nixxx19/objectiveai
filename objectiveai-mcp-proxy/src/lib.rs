//! ObjectiveAI MCP proxy library entry point. The binary at `main.rs` is
//! a thin shell over [`spawn_proxy`]; integration tests use `spawn_proxy`
//! directly so they can run the proxy in-process and tear it down via
//! the returned [`ProxyHandle`]'s `shutdown` token.

pub mod mcp;
pub mod session;
pub mod session_manager;
pub mod upstream;

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use objectiveai::mcp::Client;
use tokio_util::sync::CancellationToken;

use crate::session_manager::SessionManager;

/// Shared state every axum handler reaches via `State<AppState>`.
#[derive(Clone)]
pub struct AppState {
    pub sessions: Arc<SessionManager>,
    pub client: Arc<Client>,
}

/// Returned by [`spawn_proxy`]. Drop the `shutdown` token (or call
/// `.cancel()`) to gracefully terminate the listening server.
#[derive(Debug)]
pub struct ProxyHandle {
    /// The actual address the listener bound to.
    pub address: SocketAddr,
    /// Convenience: `http://<addr>/`.
    pub url: String,
    /// Cancel to begin graceful shutdown.
    pub shutdown: CancellationToken,
    /// The `axum::serve` task. `await` it to confirm clean exit.
    pub serve_task: tokio::task::JoinHandle<anyhow::Result<()>>,
}

/// Build the shared upstream MCP client. Defaults match
/// `Connection::new_for_test`'s backoff (500 ms initial, 1.5x multiplier,
/// 60 s max interval, 900 s elapsed budget) which has held up well in
/// the existing client-side use cases.
pub fn build_client() -> Client {
    Client::new(
        reqwest::Client::new(),
        format!("objectiveai-mcp-proxy/{}", env!("CARGO_PKG_VERSION")),
        "ObjectiveAI MCP Proxy".into(),
        "https://objectiveai.dev".into(),
        Duration::from_secs(30),       // connect_timeout
        Duration::from_millis(500),    // backoff_current_interval
        Duration::from_millis(500),    // backoff_initial_interval
        0.5,                           // backoff_randomization_factor
        1.5,                           // backoff_multiplier
        Duration::from_secs(60),       // backoff_max_interval
        Duration::from_secs(900),      // backoff_max_elapsed_time
        Duration::from_secs(30),       // call_timeout
    )
}

/// Bind the proxy on `address` and spawn its `axum::serve` future on a
/// tokio task. Returns immediately once the listener is bound.
///
/// Pass `127.0.0.1:0` for the address to let the OS pick a free port —
/// the actual port is available via [`ProxyHandle::address`].
pub async fn spawn_proxy(address: SocketAddr) -> anyhow::Result<ProxyHandle> {
    let listener = tokio::net::TcpListener::bind(address).await?;
    let bound = listener.local_addr()?;
    let url = format!("http://{bound}/");

    let state = AppState {
        sessions: Arc::new(SessionManager::new()),
        client: Arc::new(build_client()),
    };

    let shutdown = CancellationToken::new();
    let shutdown_clone = shutdown.clone();
    let router = axum::Router::new()
        .route(
            "/",
            axum::routing::post(mcp::handle_post)
                .get(mcp::handle_get)
                .delete(mcp::handle_delete),
        )
        .with_state(state);

    let serve_task = tokio::spawn(async move {
        axum::serve(listener, router)
            .with_graceful_shutdown(async move { shutdown_clone.cancelled_owned().await })
            .await
            .map_err(anyhow::Error::from)
    });

    Ok(ProxyHandle {
        address: bound,
        url,
        shutdown,
        serve_task,
    })
}
