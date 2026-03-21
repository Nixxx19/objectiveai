//! FetchSource trait — implemented by Mock, Filesystem, and GitHub.

use crate::ctx;
use objectiveai::error::ResponseError;

/// A source that can fetch individual resource definitions by path.
///
/// Implemented by Mock, Filesystem, and GitHub.
/// ObjectiveAI API does NOT implement this (it proxies to GitHub).
#[async_trait::async_trait]
pub trait Client<CTXEXT>: Send + Sync + 'static {
    async fn get_agent(
        &self,
        ctx: &ctx::Context<CTXEXT>,
        path: &objectiveai::RemotePath,
    ) -> Result<Option<objectiveai::agent::RemoteAgentBaseWithFallbacks>, ResponseError>;

    async fn get_swarm(
        &self,
        ctx: &ctx::Context<CTXEXT>,
        path: &objectiveai::RemotePath,
    ) -> Result<Option<objectiveai::swarm::RemoteSwarmBase>, ResponseError>;

    async fn get_function(
        &self,
        ctx: &ctx::Context<CTXEXT>,
        path: &objectiveai::RemotePath,
    ) -> Result<Option<objectiveai::functions::FullRemoteFunction>, ResponseError>;

    async fn get_profile(
        &self,
        ctx: &ctx::Context<CTXEXT>,
        path: &objectiveai::RemotePath,
    ) -> Result<Option<objectiveai::functions::RemoteProfile>, ResponseError>;

    /// Resolves a `RemotePathCommitOptional` to a full `RemotePath`.
    /// For sources with commits (Github, Filesystem), resolves the latest commit if missing.
    /// For Mock, returns a `RemotePath::Mock` directly.
    async fn resolve_latest(
        &self,
        ctx: &ctx::Context<CTXEXT>,
        kind: crate::retrieval::Kind,
        path: &objectiveai::RemotePathCommitOptional,
    ) -> Result<Option<objectiveai::RemotePath>, ResponseError>;
}
