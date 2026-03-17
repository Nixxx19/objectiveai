//! Filesystem list source implementation.

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
    async fn list_agents(
        &self,
        _ctx: &ctx::Context<CTXEXT>,
    ) -> Result<objectiveai::agent::response::ListAgentResponse, ResponseError> {
        Ok(objectiveai::agent::response::ListAgentResponse { data: vec![] })
    }

    async fn list_swarms(
        &self,
        _ctx: &ctx::Context<CTXEXT>,
    ) -> Result<objectiveai::swarm::response::ListSwarmResponse, ResponseError> {
        Ok(objectiveai::swarm::response::ListSwarmResponse { data: vec![] })
    }

    async fn list_functions(
        &self,
        _ctx: &ctx::Context<CTXEXT>,
    ) -> Result<objectiveai::functions::response::ListFunctionResponse, ResponseError> {
        Ok(objectiveai::functions::response::ListFunctionResponse { data: vec![] })
    }

    async fn list_profiles(
        &self,
        _ctx: &ctx::Context<CTXEXT>,
    ) -> Result<objectiveai::functions::profiles::response::ListProfileResponse, ResponseError> {
        Ok(objectiveai::functions::profiles::response::ListProfileResponse { data: vec![] })
    }

    async fn list_function_profile_pairs(
        &self,
        _ctx: &ctx::Context<CTXEXT>,
    ) -> Result<objectiveai::functions::response::ListFunctionProfilePairResponse, ResponseError>
    {
        unimplemented!("Filesystem does not support listing function-profile pairs")
    }
}
