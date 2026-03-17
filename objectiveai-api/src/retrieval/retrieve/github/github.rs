//! GitHub fetch source implementation.

use crate::ctx;
use objectiveai::error::ResponseError;
use std::sync::Arc;

pub struct GithubClient {
    pub client: Arc<crate::github::Client>,
}

impl GithubClient {
    pub fn new(client: Arc<crate::github::Client>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl<CTXEXT> super::super::Client<CTXEXT> for GithubClient
where
    CTXEXT: Send + Sync + 'static + ctx::ContextExt,
{
    async fn get_agent(
        &self,
        ctx: &ctx::Context<CTXEXT>,
        path: &objectiveai::RemotePath,
    ) -> Result<Option<objectiveai::agent::response::GetAgentResponse>, ResponseError> {
        self.client
            .read_json(ctx, &path.owner, &path.repository, &path.commit, "agent.json")
            .await
            .map(|opt| opt.map(|inner| objectiveai::agent::response::GetAgentResponse {
                path: path.clone(),
                inner,
            }))
            .map_err(|e| ResponseError::from(&e))
    }

    async fn get_swarm(
        &self,
        ctx: &ctx::Context<CTXEXT>,
        path: &objectiveai::RemotePath,
    ) -> Result<Option<objectiveai::swarm::response::GetSwarmResponse>, ResponseError> {
        self.client
            .read_json(ctx, &path.owner, &path.repository, &path.commit, "swarm.json")
            .await
            .map(|opt| opt.map(|inner| objectiveai::swarm::response::GetSwarmResponse {
                path: path.clone(),
                inner,
            }))
            .map_err(|e| ResponseError::from(&e))
    }

    async fn get_function(
        &self,
        ctx: &ctx::Context<CTXEXT>,
        path: &objectiveai::RemotePath,
    ) -> Result<Option<objectiveai::functions::FullRemoteFunction>, ResponseError> {
        self.client
            .read_json(ctx, &path.owner, &path.repository, &path.commit, "function.json")
            .await
            .map_err(|e| ResponseError::from(&e))
    }

    async fn get_profile(
        &self,
        ctx: &ctx::Context<CTXEXT>,
        path: &objectiveai::RemotePath,
    ) -> Result<Option<objectiveai::functions::RemoteProfile>, ResponseError> {
        self.client
            .read_json(ctx, &path.owner, &path.repository, &path.commit, "profile.json")
            .await
            .map_err(|e| ResponseError::from(&e))
    }

    async fn resolve_latest_commit(
        &self,
        ctx: &ctx::Context<CTXEXT>,
        _kind: crate::retrieval::Kind,
        _remote: objectiveai::Remote,
        owner: &str,
        repository: &str,
    ) -> Result<Option<String>, ResponseError> {
        self.client
            .fetch_latest_commit(ctx, owner, repository)
            .await
            .map_err(|e| ResponseError::from(&e))
    }
}
