//! Retrieve router — dispatches by `Remote`, resolves commits, and caches per request.

use crate::ctx;
use futures::FutureExt;
use objectiveai::error::ResponseError;
use objectiveai::Remote;
use std::sync::Arc;

/// Routes fetch operations by `Remote` to GitHub/Filesystem/Mock,
/// with per-request deduplication caching via context caches.
///
/// Public methods accept `RemotePathCommitOptional`. If commit is `None`,
/// the router resolves the latest commit via `resolve_latest_commit` on the
/// source trait, then delegates with a fully-resolved `RemotePath`.
pub struct Router<G, F, M> {
    pub github: Arc<G>,
    pub filesystem: Arc<F>,
    pub mock: Arc<M>,
}

impl<G, F, M> Router<G, F, M> {
    pub fn new(github: Arc<G>, filesystem: Arc<F>, mock: Arc<M>) -> Self {
        Self { github, filesystem, mock }
    }
}

impl<G, F, M, CTXEXT> Router<G, F, M>
where
    G: super::Client<CTXEXT>,
    F: super::Client<CTXEXT>,
    M: super::Client<CTXEXT>,
    CTXEXT: Send + Sync + 'static,
{
    fn source(&self, remote: Remote) -> &dyn super::Client<CTXEXT> {
        match remote {
            Remote::Github => self.github.as_ref(),
            Remote::Filesystem => self.filesystem.as_ref(),
            Remote::Mock => self.mock.as_ref(),
        }
    }

    /// Resolves a `RemotePathCommitOptional` to a `RemotePath` by looking up
    /// the latest commit if missing, with per-request dedup caching.
    async fn resolve_path(
        self: &Arc<Self>,
        ctx: &ctx::Context<CTXEXT>,
        kind: crate::retrieval::Kind,
        path: &objectiveai::RemotePathCommitOptional,
    ) -> Result<Option<objectiveai::RemotePath>, ResponseError> {
        let commit = match &path.commit {
            Some(c) => c.clone(),
            None => {
                let cache_key = (path.remote, path.owner.clone(), path.repository.clone());
                let shared = ctx
                    .latest_commit_cache
                    .entry(cache_key)
                    .or_insert_with(|| {
                        let (tx, rx) = tokio::sync::oneshot::channel();
                        let router = self.clone();
                        let remote = path.remote;
                        let owner = path.owner.clone();
                        let repository = path.repository.clone();
                        let ctx = ctx.clone();
                        tokio::spawn(async move {
                            let result = router
                                .source(remote)
                                .resolve_latest_commit(&ctx, kind, remote, &owner, &repository)
                                .await;
                            let _ = tx.send(result);
                        });
                        rx.shared()
                    })
                    .clone();
                match shared.await.unwrap()? {
                    Some(c) => c,
                    None => return Ok(None),
                }
            }
        };
        Ok(Some(objectiveai::RemotePath {
            remote: path.remote,
            owner: path.owner.clone(),
            repository: path.repository.clone(),
            commit,
        }))
    }

    /// Fetch an agent, with per-request dedup caching.
    pub async fn get_agent(
        self: &Arc<Self>,
        ctx: &ctx::Context<CTXEXT>,
        params: &objectiveai::RemotePathCommitOptional,
    ) -> Result<Option<objectiveai::agent::response::GetAgentResponse>, ResponseError> {
        let Some(path) = self.resolve_path(ctx, crate::retrieval::Kind::Agents, params).await? else {
            return Ok(None);
        };
        let shared = ctx
            .agent_cache
            .entry(path.clone())
            .or_insert_with(|| {
                let (tx, rx) = tokio::sync::oneshot::channel();
                let router = self.clone();
                let path = path.clone();
                let ctx = ctx.clone();
                tokio::spawn(async move {
                    let result = router.source(path.remote).get_agent(&ctx, &path).await;
                    let _ = tx.send(result);
                });
                rx.shared()
            })
            .clone();
        shared.await.unwrap()
    }

    /// Fetch a swarm, with per-request dedup caching.
    pub async fn get_swarm(
        self: &Arc<Self>,
        ctx: &ctx::Context<CTXEXT>,
        params: &objectiveai::RemotePathCommitOptional,
    ) -> Result<Option<objectiveai::swarm::response::GetSwarmResponse>, ResponseError> {
        let Some(path) = self.resolve_path(ctx, crate::retrieval::Kind::Swarms, params).await? else {
            return Ok(None);
        };
        let shared = ctx
            .swarm_cache
            .entry(path.clone())
            .or_insert_with(|| {
                let (tx, rx) = tokio::sync::oneshot::channel();
                let router = self.clone();
                let path = path.clone();
                let ctx = ctx.clone();
                tokio::spawn(async move {
                    let result = router.source(path.remote).get_swarm(&ctx, &path).await;
                    let _ = tx.send(result);
                });
                rx.shared()
            })
            .clone();
        shared.await.unwrap()
    }

    /// Fetch a function, with per-request dedup caching.
    pub async fn get_function(
        self: &Arc<Self>,
        ctx: &ctx::Context<CTXEXT>,
        params: &objectiveai::RemotePathCommitOptional,
    ) -> Result<Option<objectiveai::functions::FullRemoteFunction>, ResponseError> {
        let Some(path) = self.resolve_path(ctx, crate::retrieval::Kind::Functions, params).await? else {
            return Ok(None);
        };
        let cache_key = (
            path.remote,
            path.owner.clone(),
            path.repository.clone(),
            path.commit.clone(),
        );
        let shared = ctx
            .function_cache
            .entry(cache_key)
            .or_insert_with(|| {
                let (tx, rx) = tokio::sync::oneshot::channel();
                let router = self.clone();
                let path = path.clone();
                let ctx = ctx.clone();
                tokio::spawn(async move {
                    let result = router.source(path.remote).get_function(&ctx, &path).await;
                    let _ = tx.send(result);
                });
                rx.shared()
            })
            .clone();
        shared.await.unwrap()
    }

    /// Fetch an agent for the API endpoint. Returns 404 if not found.
    pub async fn endpoint_get_agent(
        self: &Arc<Self>,
        ctx: &ctx::Context<CTXEXT>,
        params: &objectiveai::RemotePathCommitOptional,
    ) -> Result<objectiveai::agent::response::GetAgentResponse, ResponseError> {
        self.get_agent(ctx, params).await?.ok_or_else(|| not_found("agent"))
    }

    /// Fetch a swarm for the API endpoint. Returns 404 if not found.
    pub async fn endpoint_get_swarm(
        self: &Arc<Self>,
        ctx: &ctx::Context<CTXEXT>,
        params: &objectiveai::RemotePathCommitOptional,
    ) -> Result<objectiveai::swarm::response::GetSwarmResponse, ResponseError> {
        self.get_swarm(ctx, params).await?.ok_or_else(|| not_found("swarm"))
    }

    /// Fetch a function and wrap into a `GetFunctionResponse` for the API endpoint. Returns 404 if not found.
    pub async fn endpoint_get_function(
        self: &Arc<Self>,
        ctx: &ctx::Context<CTXEXT>,
        params: &objectiveai::RemotePathCommitOptional,
    ) -> Result<objectiveai::functions::response::GetFunctionResponse, ResponseError> {
        let path = self.resolve_path(ctx, crate::retrieval::Kind::Functions, params).await?
            .ok_or_else(|| not_found("function"))?;
        let full_fn = self.get_function(ctx, params).await?
            .ok_or_else(|| not_found("function"))?;
        let remote_fn = full_fn.transpile();
        Ok(objectiveai::functions::response::GetFunctionResponse { path, inner: remote_fn })
    }

    /// Fetch a profile, with per-request dedup caching.
    pub async fn get_profile(
        self: &Arc<Self>,
        ctx: &ctx::Context<CTXEXT>,
        params: &objectiveai::RemotePathCommitOptional,
    ) -> Result<Option<objectiveai::functions::RemoteProfile>, ResponseError> {
        let Some(path) = self.resolve_path(ctx, crate::retrieval::Kind::Profiles, params).await? else {
            return Ok(None);
        };
        let cache_key = (
            path.remote,
            path.owner.clone(),
            path.repository.clone(),
            path.commit.clone(),
        );
        let shared = ctx
            .profile_cache
            .entry(cache_key)
            .or_insert_with(|| {
                let (tx, rx) = tokio::sync::oneshot::channel();
                let router = self.clone();
                let path = path.clone();
                let ctx = ctx.clone();
                tokio::spawn(async move {
                    let result = router.source(path.remote).get_profile(&ctx, &path).await;
                    let _ = tx.send(result);
                });
                rx.shared()
            })
            .clone();
        shared.await.unwrap()
    }

    /// Fetch a profile and wrap into a `GetProfileResponse` for the API endpoint. Returns 404 if not found.
    pub async fn endpoint_get_profile(
        self: &Arc<Self>,
        ctx: &ctx::Context<CTXEXT>,
        params: &objectiveai::RemotePathCommitOptional,
    ) -> Result<objectiveai::functions::profiles::response::GetProfileResponse, ResponseError> {
        let path = self.resolve_path(ctx, crate::retrieval::Kind::Profiles, params).await?
            .ok_or_else(|| not_found("profile"))?;
        let profile = self.get_profile(ctx, params).await?
            .ok_or_else(|| not_found("profile"))?;
        Ok(objectiveai::functions::profiles::response::GetProfileResponse { path, inner: profile })
    }
}

fn not_found(kind: &str) -> ResponseError {
    ResponseError {
        code: 404,
        message: serde_json::json!({ "error": format!("{} not found", kind) }),
    }
}
