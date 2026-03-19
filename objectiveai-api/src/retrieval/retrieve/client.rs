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

    /// Resolves the latest commit for a given owner/repository.
    /// Used by the router when `RemotePathCommitOptional` has no commit.
    async fn resolve_latest_commit(
        &self,
        ctx: &ctx::Context<CTXEXT>,
        kind: crate::retrieval::Kind,
        remote: objectiveai::Remote,
        owner: &str,
        repository: &str,
    ) -> Result<Option<String>, ResponseError>;
}
