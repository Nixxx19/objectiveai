//! Filesystem fetch source implementation.

use crate::ctx;
use objectiveai::error::ResponseError;
use std::sync::Arc;

pub struct FilesystemClient {
    pub client: Arc<crate::filesystem::Client>,
}

impl FilesystemClient {
    pub fn new(client: Arc<crate::filesystem::Client>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl<CTXEXT> super::super::Client<CTXEXT> for FilesystemClient
where
    CTXEXT: Send + Sync + 'static,
{
    async fn get_agent(
        &self,
        _ctx: &ctx::Context<CTXEXT>,
        path: &objectiveai::RemotePath,
    ) -> Result<Option<objectiveai::agent::response::GetAgentResponse>, ResponseError> {
        match self
            .client
            .read_json::<objectiveai::agent::RemoteAgent>(
                crate::retrieval::Kind::Agents,
                &path.owner,
                &path.repository,
                Some(&path.commit),
                "agent.json",
            )
            .await
        {
            Ok(Some((agent, _resolved_commit))) => Ok(Some(objectiveai::agent::response::GetAgentResponse {
                path: path.clone(),
                inner: agent,
            })),
            Ok(None) => Ok(None),
            Err(e) => Err(ResponseError::from(&e)),
        }
    }

    async fn get_swarm(
        &self,
        _ctx: &ctx::Context<CTXEXT>,
        path: &objectiveai::RemotePath,
    ) -> Result<Option<objectiveai::swarm::response::GetSwarmResponse>, ResponseError> {
        match self
            .client
            .read_json::<objectiveai::swarm::RemoteSwarm>(
                crate::retrieval::Kind::Swarms,
                &path.owner,
                &path.repository,
                Some(&path.commit),
                "swarm.json",
            )
            .await
        {
            Ok(Some((swarm, _resolved_commit))) => Ok(Some(objectiveai::swarm::response::GetSwarmResponse {
                path: path.clone(),
                inner: swarm,
            })),
            Ok(None) => Ok(None),
            Err(e) => Err(ResponseError::from(&e)),
        }
    }

    async fn get_function(
        &self,
        _ctx: &ctx::Context<CTXEXT>,
        path: &objectiveai::RemotePath,
    ) -> Result<Option<objectiveai::functions::FullRemoteFunction>, ResponseError> {
        match self
            .client
            .read_json::<objectiveai::functions::FullRemoteFunction>(
                crate::retrieval::Kind::Functions,
                &path.owner,
                &path.repository,
                Some(&path.commit),
                "function.json",
            )
            .await
        {
            Ok(Some((function, _resolved_commit))) => Ok(Some(function)),
            Ok(None) => Ok(None),
            Err(e) => Err(ResponseError::from(&e)),
        }
    }

    async fn get_profile(
        &self,
        _ctx: &ctx::Context<CTXEXT>,
        path: &objectiveai::RemotePath,
    ) -> Result<Option<objectiveai::functions::RemoteProfile>, ResponseError> {
        match self
            .client
            .read_json::<objectiveai::functions::RemoteProfile>(
                crate::retrieval::Kind::Profiles,
                &path.owner,
                &path.repository,
                Some(&path.commit),
                "profile.json",
            )
            .await
        {
            Ok(Some((profile, _resolved_commit))) => Ok(Some(profile)),
            Ok(None) => Ok(None),
            Err(e) => Err(ResponseError::from(&e)),
        }
    }

    async fn resolve_latest_commit(
        &self,
        _ctx: &ctx::Context<CTXEXT>,
        kind: crate::retrieval::Kind,
        _remote: objectiveai::Remote,
        owner: &str,
        repository: &str,
    ) -> Result<Option<String>, ResponseError> {
        match self.client.resolve_head(kind, owner, repository) {
            Ok(commit) => Ok(Some(commit)),
            Err(_) => Ok(None),
        }
    }
}
