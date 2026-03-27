//! FetchSource trait — implemented by Mock, Filesystem, and GitHub.

use crate::ctx;
use objectiveai::error::ResponseError;

/// A source that can fetch individual resource definitions by path.
///
/// Implemented by Mock, Filesystem, and GitHub.
/// ObjectiveAI API does NOT implement this (it proxies to GitHub).
#[async_trait::async_trait]
pub trait Client<CTXEXT>: Send + Sync + 'static {
    async fn get_agent<PC: crate::ctx::persistent_cache::PersistentCacheClient>(
        &self,
        ctx: &ctx::Context<CTXEXT, PC>,
        path: &objectiveai::RemotePath,
    ) -> Result<Option<objectiveai::agent::RemoteAgentBaseWithFallbacks>, ResponseError>;

    async fn get_swarm<PC: crate::ctx::persistent_cache::PersistentCacheClient>(
        &self,
        ctx: &ctx::Context<CTXEXT, PC>,
        path: &objectiveai::RemotePath,
    ) -> Result<Option<objectiveai::swarm::RemoteSwarmBase>, ResponseError>;

    async fn get_function<PC: crate::ctx::persistent_cache::PersistentCacheClient>(
        &self,
        ctx: &ctx::Context<CTXEXT, PC>,
        path: &objectiveai::RemotePath,
    ) -> Result<Option<objectiveai::functions::FullRemoteFunction>, ResponseError>;

    async fn get_profile<PC: crate::ctx::persistent_cache::PersistentCacheClient>(
        &self,
        ctx: &ctx::Context<CTXEXT, PC>,
        path: &objectiveai::RemotePath,
    ) -> Result<Option<objectiveai::functions::RemoteProfile>, ResponseError>;

    /// Resolves a `RemotePathCommitOptional` to a full `RemotePath`.
    /// For sources with commits (Github, Filesystem), resolves the latest commit if missing.
    /// For Mock, returns a `RemotePath::Mock` directly.
    async fn resolve_latest<PC: crate::ctx::persistent_cache::PersistentCacheClient>(
        &self,
        ctx: &ctx::Context<CTXEXT, PC>,
        kind: crate::retrieval::Kind,
        path: &objectiveai::RemotePathCommitOptional,
    ) -> Result<Option<objectiveai::RemotePath>, ResponseError>;
}
