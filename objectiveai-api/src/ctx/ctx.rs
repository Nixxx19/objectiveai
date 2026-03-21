//! Request context containing per-request state and caches.

use dashmap::DashMap;
use futures::future::Shared;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::OnceCell;

/// Per-request context containing user-specific state and deduplication caches.
///
/// The context is generic over `CTXEXT`, allowing custom extensions for
/// different deployment scenarios (e.g., different BYOK providers).
///
/// # Caches
///
/// The caches deduplicate concurrent fetches for the same resource within a request.
/// When multiple parts of a request need the same swarm or agent,
/// only one fetch is performed and the result is shared.
#[derive(Debug)]
pub struct Context<CTXEXT> {
    /// Custom context extension (e.g., for BYOK keys).
    pub ext: Arc<CTXEXT>,
    /// Multiplier applied to costs for this request.
    pub cost_multiplier: rust_decimal::Decimal,
    /// Per-request OpenRouter authorization token.
    openrouter_authorization: Option<Arc<String>>,
    /// Per-request GitHub authorization token.
    github_authorization: Option<Arc<String>>,
    /// Per-request MCP authorization headers.
    mcp_authorization: Option<Arc<HashMap<String, String>>>,
    /// Cached resolved OpenRouter authorization (self + BYOK).
    openrouter_authorization_cached: OnceCell<Option<Arc<String>>>,
    /// Cached resolved GitHub authorization (self + BYOK).
    github_authorization_cached: OnceCell<Option<Arc<String>>>,
    /// Cached resolved MCP authorization (self + BYOK merged).
    mcp_authorization_cached: OnceCell<Option<Arc<HashMap<String, String>>>>,
    /// Cache for agent fetches, keyed by RemotePath.
    pub agent_cache: Arc<
        DashMap<
            objectiveai::RemotePath,
            Shared<
                tokio::sync::oneshot::Receiver<
                    Result<
                        Option<objectiveai::agent::RemoteAgentBaseWithFallbacks>,
                        objectiveai::error::ResponseError,
                    >,
                >,
            >,
        >,
    >,
    /// Cache for swarm fetches, keyed by RemotePath.
    pub swarm_cache: Arc<
        DashMap<
            objectiveai::RemotePath,
            Shared<
                tokio::sync::oneshot::Receiver<
                    Result<
                        Option<objectiveai::swarm::RemoteSwarmBase>,
                        objectiveai::error::ResponseError,
                    >,
                >,
            >,
        >,
    >,
    /// Cache for function fetches, keyed by RemotePath.
    pub function_cache: Arc<
        DashMap<
            objectiveai::RemotePath,
            Shared<
                tokio::sync::oneshot::Receiver<
                    Result<
                        Option<objectiveai::functions::FullRemoteFunction>,
                        objectiveai::error::ResponseError,
                    >,
                >,
            >,
        >,
    >,
    /// Cache for profile fetches, keyed by RemotePath.
    pub profile_cache: Arc<
        DashMap<
            objectiveai::RemotePath,
            Shared<
                tokio::sync::oneshot::Receiver<
                    Result<
                        Option<objectiveai::functions::RemoteProfile>,
                        objectiveai::error::ResponseError,
                    >,
                >,
            >,
        >,
    >,
    /// Cache for resolve_latest fetches, keyed by RemotePathCommitOptional.
    pub remote_latest_cache: Arc<
        DashMap<
            objectiveai::RemotePathCommitOptional,
            Shared<
                tokio::sync::oneshot::Receiver<
                    Result<
                        Option<objectiveai::RemotePath>,
                        objectiveai::error::ResponseError,
                    >,
                >,
            >,
        >,
    >,
}

impl<CTXEXT> Clone for Context<CTXEXT> {
    fn clone(&self) -> Self {
        Self {
            ext: self.ext.clone(),
            cost_multiplier: self.cost_multiplier,
            openrouter_authorization: self.openrouter_authorization.clone(),
            github_authorization: self.github_authorization.clone(),
            mcp_authorization: self.mcp_authorization.clone(),
            openrouter_authorization_cached: OnceCell::new(),
            github_authorization_cached: OnceCell::new(),
            mcp_authorization_cached: OnceCell::new(),
            swarm_cache: self.swarm_cache.clone(),
            agent_cache: self.agent_cache.clone(),
            remote_latest_cache: self.remote_latest_cache.clone(),
            function_cache: self.function_cache.clone(),
            profile_cache: self.profile_cache.clone(),
        }
    }
}

impl<CTXEXT> Context<CTXEXT> {
    /// Creates a new context by extracting authorization headers from the request.
    ///
    /// For each header, checks the `X-` prefixed variant first, then falls back
    /// to the non-prefixed variant:
    /// - `X-OPENROUTER-AUTHORIZATION` / `OPENROUTER-AUTHORIZATION`: OpenRouter API key
    /// - `X-GITHUB-AUTHORIZATION` / `GITHUB-AUTHORIZATION`: GitHub token
    /// - `X-MCP-AUTHORIZATION` / `MCP-AUTHORIZATION`: JSON-encoded `HashMap<String, String>`
    pub fn new(
        ext: Arc<CTXEXT>,
        cost_multiplier: rust_decimal::Decimal,
        headers: &axum::http::HeaderMap,
    ) -> Self {
        let openrouter_authorization = headers
            .get("X-OPENROUTER-AUTHORIZATION")
            .or_else(|| headers.get("OPENROUTER-AUTHORIZATION"))
            .and_then(|v| v.to_str().ok())
            .map(|s| Arc::new(s.to_owned()));

        let github_authorization = headers
            .get("X-GITHUB-AUTHORIZATION")
            .or_else(|| headers.get("GITHUB-AUTHORIZATION"))
            .and_then(|v| v.to_str().ok())
            .map(|s| Arc::new(s.to_owned()));

        let mcp_authorization = headers
            .get("X-MCP-AUTHORIZATION")
            .or_else(|| headers.get("MCP-AUTHORIZATION"))
            .and_then(|v| v.to_str().ok())
            .and_then(|s| serde_json::from_str::<HashMap<String, String>>(s).ok())
            .map(Arc::new);

        Self {
            ext,
            cost_multiplier,
            openrouter_authorization,
            github_authorization,
            mcp_authorization,
            openrouter_authorization_cached: OnceCell::new(),
            github_authorization_cached: OnceCell::new(),
            mcp_authorization_cached: OnceCell::new(),
            swarm_cache: Arc::new(DashMap::new()),
            agent_cache: Arc::new(DashMap::new()),
            remote_latest_cache: Arc::new(DashMap::new()),
            function_cache: Arc::new(DashMap::new()),
            profile_cache: Arc::new(DashMap::new()),
        }
    }
}

impl<CTXEXT: super::ContextExt> Context<CTXEXT> {
    /// Returns the resolved upstream BYOK API key.
    ///
    /// Only OpenRouter is supported. Returns `None` for other upstreams.
    /// Checks the per-request token first, falls back to the BYOK token
    /// from the context extension. Result is cached for subsequent calls.
    pub async fn get_upstream_byok(
        &self,
        upstream: objectiveai::agent::Upstream,
    ) -> Option<Arc<String>> {
        if upstream != objectiveai::agent::Upstream::Openrouter {
            return None;
        }
        self.openrouter_authorization_cached
            .get_or_init(|| async {
                match (&self.openrouter_authorization, self.ext.get_openrouter_byok().await) {
                    (Some(self_token), _) => Some(self_token.clone()),
                    (None, byok) => byok,
                }
            })
            .await
            .clone()
    }

    /// Returns the resolved GitHub authorization token.
    ///
    /// Checks the per-request token first, falls back to the BYOK token
    /// from the context extension. Result is cached for subsequent calls.
    pub async fn github_authorization(&self) -> Option<Arc<String>> {
        self.github_authorization_cached
            .get_or_init(|| async {
                match (&self.github_authorization, self.ext.get_github_byok().await) {
                    (Some(self_token), _) => Some(self_token.clone()),
                    (None, byok) => byok,
                }
            })
            .await
            .clone()
    }

    /// Returns the resolved MCP authorization headers.
    ///
    /// Merges the per-request headers with BYOK headers from the context
    /// extension. Per-request headers take priority over BYOK headers.
    /// Result is cached for subsequent calls.
    pub async fn mcp_authorization(&self) -> Option<Arc<HashMap<String, String>>> {
        self.mcp_authorization_cached
            .get_or_init(|| async {
                let byok = self.ext.get_mcp_byok().await;
                match (&self.mcp_authorization, byok) {
                    (None, None) => None,
                    (Some(self_headers), None) => Some(self_headers.clone()),
                    (None, Some(byok_headers)) => Some(byok_headers),
                    (Some(self_headers), Some(byok_headers)) => {
                        let mut merged = (*byok_headers).clone();
                        for (k, v) in self_headers.iter() {
                            merged.insert(k.clone(), v.clone());
                        }
                        Some(Arc::new(merged))
                    }
                }
            })
            .await
            .clone()
    }
}
