//! Retrieve router — dispatches by `Remote`, resolves commits, and caches per request.

use crate::ctx;
use futures::FutureExt;
use objectiveai::error::ResponseError;
use objectiveai::Remote;
use std::sync::Arc;

/// Routes fetch operations by `Remote` to GitHub/Filesystem/Mock,
/// with per-request deduplication caching via context caches.
///
/// Main methods accept `CommitOptional` enums (inline or remote ref).
/// If inline, converts directly. If remote, resolves commit, fetches
/// from source, converts, and returns the union type.
pub struct Router<G, F, M, CTXEXT> {
    pub github: Arc<G>,
    pub filesystem: Arc<F>,
    pub mock: Arc<M>,
    _ctxext: std::marker::PhantomData<CTXEXT>,
}

impl<G, F, M, CTXEXT> Router<G, F, M, CTXEXT> {
    pub fn new(github: Arc<G>, filesystem: Arc<F>, mock: Arc<M>) -> Self {
        Self { github, filesystem, mock, _ctxext: std::marker::PhantomData }
    }
}


impl<G, F, M, CTXEXT> Router<G, F, M, CTXEXT>
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

    /// Resolves a `RemotePathCommitOptional` to a `RemotePath`.
    pub async fn resolve_path(
        self: &Arc<Self>,
        ctx: &ctx::Context<CTXEXT>,
        kind: crate::retrieval::Kind,
        path: &objectiveai::RemotePathCommitOptional,
    ) -> Result<Option<objectiveai::RemotePath>, ResponseError> {
        let remote = path.remote();
        let cache_key = path.clone();
        let path_clone = path.clone();
        let shared = ctx
            .remote_latest_cache
            .entry(cache_key)
            .or_insert_with(|| {
                let (tx, rx) = tokio::sync::oneshot::channel();
                let router = self.clone();
                let ctx = ctx.clone();
                tokio::spawn(async move {
                    let result = router
                        .source(remote)
                        .resolve_latest(&ctx, kind, &path_clone)
                        .await;
                    let _ = tx.send(result);
                });
                rx.shared()
            })
            .clone();
        shared.await.unwrap()
    }

    // ── Agent ──────────────────────────────────────────────────────

    /// Resolve an agent: inline converts directly, remote fetches and converts.
    pub async fn get_agent(
        self: &Arc<Self>,
        ctx: &ctx::Context<CTXEXT>,
        params: objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional,
    ) -> Result<objectiveai::agent::AgentWithFallbacks, ResponseError> {
        match params {
            objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(base) => {
                let converted = base.convert().map_err(|e| bad_request(&e))?;
                Ok(objectiveai::agent::AgentWithFallbacks::Inline(converted))
            }
            objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::Remote(remote) => {
                let base = self.fetch_agent_base(ctx, &remote).await?
                    .ok_or_else(|| not_found("agent"))?;
                let converted = base.convert().map_err(|e| bad_request(&e))?;
                Ok(objectiveai::agent::AgentWithFallbacks::Remote(converted))
            }
        }
    }

    /// Fetch a raw `RemoteAgentBaseWithFallbacks` from a source, with per-request dedup caching.
    async fn fetch_agent_base(
        self: &Arc<Self>,
        ctx: &ctx::Context<CTXEXT>,
        params: &objectiveai::RemotePathCommitOptional,
    ) -> Result<Option<objectiveai::agent::RemoteAgentBaseWithFallbacks>, ResponseError> {
        let Some(path) = self.resolve_path(ctx, crate::retrieval::Kind::Agents, params).await? else {
            return Ok(None);
        };
        let shared = ctx
            .agent_cache
            .entry(path.clone())
            .or_insert_with(|| {
                let (tx, rx) = tokio::sync::oneshot::channel();
                let router = self.clone();
                let remote = path.remote();
                let path = path.clone();
                let ctx = ctx.clone();
                tokio::spawn(async move {
                    let result = router.source(remote).get_agent(&ctx, &path).await;
                    let _ = tx.send(result);
                });
                rx.shared()
            })
            .clone();
        shared.await.unwrap()
    }

    /// API endpoint: fetch a remote agent, convert, wrap in response.
    pub async fn endpoint_get_agent(
        self: &Arc<Self>,
        ctx: &ctx::Context<CTXEXT>,
        params: &objectiveai::RemotePathCommitOptional,
    ) -> Result<objectiveai::agent::response::GetAgentResponse, ResponseError> {
        let path = self.resolve_path(ctx, crate::retrieval::Kind::Agents, params).await?
            .ok_or_else(|| not_found("agent"))?;
        let result = self.get_agent(
            ctx,
            objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::Remote(params.clone()),
        ).await?;
        let inner = match result {
            objectiveai::agent::AgentWithFallbacks::Remote(r) => r,
            objectiveai::agent::AgentWithFallbacks::Inline(_) => unreachable!(),
        };
        Ok(objectiveai::agent::response::GetAgentResponse { path, inner })
    }

    // ── Swarm ─────────────────────────────────────────────────────

    /// Resolve a swarm: inline converts directly, remote fetches and converts.
    /// Remote agent references in the swarm's agents list are resolved automatically.
    pub async fn get_swarm(
        self: &Arc<Self>,
        ctx: &ctx::Context<CTXEXT>,
        params: objectiveai::swarm::InlineSwarmBaseOrRemoteCommitOptional,
    ) -> Result<objectiveai::swarm::Swarm, ResponseError> {
        match params {
            objectiveai::swarm::InlineSwarmBaseOrRemoteCommitOptional::SwarmBase(base) => {
                let converted = self.resolve_swarm_base(ctx, base).await?;
                Ok(objectiveai::swarm::Swarm::Inline(converted))
            }
            objectiveai::swarm::InlineSwarmBaseOrRemoteCommitOptional::Remote(remote) => {
                let base = self.fetch_swarm_base(ctx, &remote).await?
                    .ok_or_else(|| not_found("swarm"))?;
                let converted = self.resolve_swarm_base(ctx, base.inner).await?;
                Ok(objectiveai::swarm::Swarm::Remote(objectiveai::swarm::RemoteSwarm {
                    description: base.description,
                    inner: converted,
                }))
            }
        }
    }

    /// Resolve remote agent references in a swarm base and convert it.
    /// All remote agents are fetched concurrently.
    async fn resolve_swarm_base(
        self: &Arc<Self>,
        ctx: &ctx::Context<CTXEXT>,
        base: objectiveai::swarm::InlineSwarmBase,
    ) -> Result<objectiveai::swarm::InlineSwarm, ResponseError> {
        // Collect unique remote agent paths to fetch.
        let mut unique_paths: indexmap::IndexMap<String, objectiveai::RemotePathCommitOptional> =
            indexmap::IndexMap::new();
        for agent_slot in &base.agents {
            if let objectiveai::agent::InlineAgentBaseWithFallbacksOrRemote::Remote(path) =
                &agent_slot.inner
            {
                let key = path.key();
                unique_paths
                    .entry(key)
                    .or_insert_with(|| path.clone().into());
            }
        }

        // Fetch all remote agents concurrently.
        if !unique_paths.is_empty() {
            let futs: Vec<_> = unique_paths
                .iter()
                .map(|(key, path)| {
                    let key = key.clone();
                    async move {
                        let agent_base = self.fetch_agent_base(ctx, path).await?
                            .ok_or_else(|| not_found("agent"))?;
                        Ok::<_, ResponseError>((key, agent_base))
                    }
                })
                .collect();
            let results = futures::future::try_join_all(futs).await?;
            let remote_agents: std::collections::HashMap<_, _> = results.into_iter().collect();
            base.convert(Some(&remote_agents)).map_err(|e| bad_request(&e))
        } else {
            base.convert(None).map_err(|e| bad_request(&e))
        }
    }

    /// Fetch a raw `RemoteSwarmBase` from a source, with per-request dedup caching.
    /// Falls back to swarm.json if profile.json is not found (for profile retrieval).
    async fn fetch_swarm_base(
        self: &Arc<Self>,
        ctx: &ctx::Context<CTXEXT>,
        params: &objectiveai::RemotePathCommitOptional,
    ) -> Result<Option<objectiveai::swarm::RemoteSwarmBase>, ResponseError> {
        let Some(path) = self.resolve_path(ctx, crate::retrieval::Kind::Swarms, params).await? else {
            return Ok(None);
        };
        let shared = ctx
            .swarm_cache
            .entry(path.clone())
            .or_insert_with(|| {
                let (tx, rx) = tokio::sync::oneshot::channel();
                let router = self.clone();
                let remote = path.remote();
                let path = path.clone();
                let ctx = ctx.clone();
                tokio::spawn(async move {
                    let result = router.source(remote).get_swarm(&ctx, &path).await;
                    let _ = tx.send(result);
                });
                rx.shared()
            })
            .clone();
        shared.await.unwrap()
    }

    /// API endpoint: fetch a remote swarm, convert, wrap in response.
    pub async fn endpoint_get_swarm(
        self: &Arc<Self>,
        ctx: &ctx::Context<CTXEXT>,
        params: &objectiveai::RemotePathCommitOptional,
    ) -> Result<objectiveai::swarm::response::GetSwarmResponse, ResponseError> {
        let path = self.resolve_path(ctx, crate::retrieval::Kind::Swarms, params).await?
            .ok_or_else(|| not_found("swarm"))?;
        let result = self.get_swarm(
            ctx,
            objectiveai::swarm::InlineSwarmBaseOrRemoteCommitOptional::Remote(params.clone()),
        ).await?;
        let inner = match result {
            objectiveai::swarm::Swarm::Remote(r) => r,
            objectiveai::swarm::Swarm::Inline(_) => unreachable!(),
        };
        Ok(objectiveai::swarm::response::GetSwarmResponse { path, inner })
    }

    // ── Function ──────────────────────────────────────────────────

    /// Resolve a function: inline returns directly, remote fetches with caching.
    pub async fn get_function(
        self: &Arc<Self>,
        ctx: &ctx::Context<CTXEXT>,
        params: objectiveai::functions::FullInlineFunctionOrRemoteCommitOptional,
    ) -> Result<objectiveai::functions::FullFunction, ResponseError> {
        match params {
            objectiveai::functions::FullInlineFunctionOrRemoteCommitOptional::Inline(inline) => {
                Ok(objectiveai::functions::FullFunction::Inline(inline))
            }
            objectiveai::functions::FullInlineFunctionOrRemoteCommitOptional::Remote(remote) => {
                let fetched = self.fetch_function(ctx, &remote).await?
                    .ok_or_else(|| not_found("function"))?;
                Ok(objectiveai::functions::FullFunction::Remote(fetched))
            }
        }
    }

    /// Fetch a raw `FullRemoteFunction` from a source, with per-request dedup caching.
    async fn fetch_function(
        self: &Arc<Self>,
        ctx: &ctx::Context<CTXEXT>,
        params: &objectiveai::RemotePathCommitOptional,
    ) -> Result<Option<objectiveai::functions::FullRemoteFunction>, ResponseError> {
        let Some(path) = self.resolve_path(ctx, crate::retrieval::Kind::Functions, params).await? else {
            return Ok(None);
        };
        let cache_key = path.clone();
        let shared = ctx
            .function_cache
            .entry(cache_key)
            .or_insert_with(|| {
                let (tx, rx) = tokio::sync::oneshot::channel();
                let router = self.clone();
                let remote = path.remote();
                let path = path.clone();
                let ctx = ctx.clone();
                tokio::spawn(async move {
                    let result = router.source(remote).get_function(&ctx, &path).await;
                    let _ = tx.send(result);
                });
                rx.shared()
            })
            .clone();
        shared.await.unwrap()
    }

    /// API endpoint: fetch a remote function, wrap in response.
    pub async fn endpoint_get_function(
        self: &Arc<Self>,
        ctx: &ctx::Context<CTXEXT>,
        params: &objectiveai::RemotePathCommitOptional,
    ) -> Result<objectiveai::functions::response::GetFunctionResponse, ResponseError> {
        let path = self.resolve_path(ctx, crate::retrieval::Kind::Functions, params).await?
            .ok_or_else(|| not_found("function"))?;
        let result = self.get_function(
            ctx,
            objectiveai::functions::FullInlineFunctionOrRemoteCommitOptional::Remote(params.clone()),
        ).await?;
        let inner = match result {
            objectiveai::functions::FullFunction::Remote(r) => r,
            objectiveai::functions::FullFunction::Inline(_) => unreachable!(),
        };
        Ok(objectiveai::functions::response::GetFunctionResponse { path, inner })
    }

    /// Recursively fetches all child functions referenced by a function's tasks.
    ///
    /// Iterates over the function's task expressions, finds ScalarFunction and
    /// VectorFunction tasks (which reference remote functions), and fetches each
    /// concurrently. Returns a HashMap keyed by the path's key string.
    pub async fn get_function_recursive(
        self: &Arc<Self>,
        ctx: &ctx::Context<CTXEXT>,
        function: objectiveai::functions::FullRemoteFunction,
    ) -> Result<std::collections::HashMap<String, objectiveai::functions::FullRemoteFunction>, ResponseError> {
        let transpiled = function.transpile();
        let mut futs: Vec<(String, _)> = Vec::new();

        for task_expr in transpiled.tasks() {
            let path = match task_expr {
                objectiveai::functions::TaskExpression::ScalarFunction(t) => {
                    t.path.clone()
                }
                objectiveai::functions::TaskExpression::VectorFunction(t) => {
                    t.path.clone()
                }
                _ => continue,
            };
            let key = path.key();
            let params = objectiveai::functions::FullInlineFunctionOrRemoteCommitOptional::Remote(
                path.into(),
            );
            let router = self.clone();
            let ctx = ctx.clone();
            futs.push((key, tokio::spawn(async move {
                router.get_function(&ctx, params).await
            })));
        }

        let mut children = std::collections::HashMap::new();
        for (key, handle) in futs {
            let full_fn = handle.await.expect("get_function task panicked")?;
            match full_fn {
                objectiveai::functions::FullFunction::Remote(r) => {
                    children.insert(key, r);
                }
                objectiveai::functions::FullFunction::Inline(_) => {
                    unreachable!()
                }
            }
        }

        Ok(children)
    }

    // ── Profile ───────────────────────────────────────────────────

    /// Resolve a profile: inline returns directly, remote fetches with caching.
    pub async fn get_profile(
        self: &Arc<Self>,
        ctx: &ctx::Context<CTXEXT>,
        params: objectiveai::functions::InlineProfileOrRemoteCommitOptional,
    ) -> Result<objectiveai::functions::Profile, ResponseError> {
        match params {
            objectiveai::functions::InlineProfileOrRemoteCommitOptional::Inline(inline) => {
                Ok(objectiveai::functions::Profile::Inline(inline))
            }
            objectiveai::functions::InlineProfileOrRemoteCommitOptional::Remote(remote) => {
                let fetched = self.fetch_profile(ctx, &remote).await?
                    .ok_or_else(|| not_found("profile"))?;
                Ok(objectiveai::functions::Profile::Remote(fetched))
            }
        }
    }

    /// Fetch a raw `RemoteProfile` from a source, with per-request dedup caching.
    /// Falls back to swarm.json if profile.json is not found.
    async fn fetch_profile(
        self: &Arc<Self>,
        ctx: &ctx::Context<CTXEXT>,
        params: &objectiveai::RemotePathCommitOptional,
    ) -> Result<Option<objectiveai::functions::RemoteProfile>, ResponseError> {
        let Some(path) = self.resolve_path(ctx, crate::retrieval::Kind::Profiles, params).await? else {
            return Ok(None);
        };
        let cache_key = path.clone();
        let shared = ctx
            .profile_cache
            .entry(cache_key)
            .or_insert_with(|| {
                let (tx, rx) = tokio::sync::oneshot::channel();
                let router = self.clone();
                let remote = path.remote();
                let path = path.clone();
                let ctx = ctx.clone();
                tokio::spawn(async move {
                    // Try profile.json first
                    let result = router.source(remote).get_profile(&ctx, &path).await;
                    let result = match &result {
                        Ok(None) => {
                            // Fallback: try swarm.json (a swarm definition is a valid Auto profile)
                            match router.source(remote).get_swarm(&ctx, &path).await {
                                Ok(Some(swarm)) => Ok(Some(
                                    objectiveai::functions::RemoteProfile::Auto(swarm),
                                )),
                                Ok(None) => Ok(None),
                                Err(e) => Err(e),
                            }
                        }
                        _ => result,
                    };
                    let _ = tx.send(result);
                });
                rx.shared()
            })
            .clone();
        shared.await.unwrap()
    }

    /// API endpoint: fetch a remote profile, wrap in response.
    pub async fn endpoint_get_profile(
        self: &Arc<Self>,
        ctx: &ctx::Context<CTXEXT>,
        params: &objectiveai::RemotePathCommitOptional,
    ) -> Result<objectiveai::functions::profiles::response::GetProfileResponse, ResponseError> {
        let path = self.resolve_path(ctx, crate::retrieval::Kind::Profiles, params).await?
            .ok_or_else(|| not_found("profile"))?;
        let result = self.get_profile(
            ctx,
            objectiveai::functions::InlineProfileOrRemoteCommitOptional::Remote(params.clone()),
        ).await?;
        let inner = match result {
            objectiveai::functions::Profile::Remote(r) => r,
            objectiveai::functions::Profile::Inline(_) => unreachable!(),
        };
        Ok(objectiveai::functions::profiles::response::GetProfileResponse { path, inner })
    }
}

fn not_found(kind: &str) -> ResponseError {
    ResponseError {
        code: 404,
        message: serde_json::json!({ "error": format!("{} not found", kind) }),
    }
}

fn bad_request(msg: &str) -> ResponseError {
    ResponseError {
        code: 400,
        message: serde_json::json!({ "error": msg }),
    }
}
