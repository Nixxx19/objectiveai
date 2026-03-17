//! Mock fetch source implementation.

use crate::ctx;
use objectiveai::error::ResponseError;

pub struct MockClient;

#[async_trait::async_trait]
impl<CTXEXT> super::super::Client<CTXEXT> for MockClient
where
    CTXEXT: Send + Sync + 'static,
{
    async fn get_agent(
        &self,
        _ctx: &ctx::Context<CTXEXT>,
        path: &objectiveai::RemotePath,
    ) -> Result<Option<objectiveai::agent::response::GetAgentResponse>, ResponseError> {
        Ok(crate::functions::mock::get_agent(
            &path.owner,
            &path.repository,
            Some(&path.commit),
        ).map(|inner| objectiveai::agent::response::GetAgentResponse {
            path: path.clone(),
            inner,
        }))
    }

    async fn get_swarm(
        &self,
        _ctx: &ctx::Context<CTXEXT>,
        path: &objectiveai::RemotePath,
    ) -> Result<Option<objectiveai::swarm::response::GetSwarmResponse>, ResponseError> {
        Ok(crate::functions::mock::get_swarm(
            &path.owner,
            &path.repository,
            Some(&path.commit),
        ).map(|inner| objectiveai::swarm::response::GetSwarmResponse {
            path: path.clone(),
            inner,
        }))
    }

    async fn get_function(
        &self,
        _ctx: &ctx::Context<CTXEXT>,
        path: &objectiveai::RemotePath,
    ) -> Result<Option<objectiveai::functions::FullRemoteFunction>, ResponseError> {
        Ok(crate::functions::mock::get_function(
            &path.owner,
            &path.repository,
            Some(&path.commit),
        ))
    }

    async fn get_profile(
        &self,
        _ctx: &ctx::Context<CTXEXT>,
        path: &objectiveai::RemotePath,
    ) -> Result<Option<objectiveai::functions::RemoteProfile>, ResponseError> {
        Ok(crate::functions::mock::get_profile(
            &path.owner,
            &path.repository,
            Some(&path.commit),
        ))
    }

    async fn resolve_latest_commit(
        &self,
        _ctx: &ctx::Context<CTXEXT>,
        _kind: crate::retrieval::Kind,
        _remote: objectiveai::Remote,
        owner: &str,
        _repository: &str,
    ) -> Result<Option<String>, ResponseError> {
        if owner == "mock" {
            Ok(Some("mock".to_string()))
        } else {
            Ok(None)
        }
    }
}
