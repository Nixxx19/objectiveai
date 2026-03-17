//! Mock list source implementation.

use crate::ctx;
use objectiveai::error::ResponseError;

pub struct MockClient;

#[async_trait::async_trait]
impl<CTXEXT> super::super::Client<CTXEXT> for MockClient
where
    CTXEXT: Send + Sync + 'static,
{
    async fn list_agents(
        &self,
        _ctx: &ctx::Context<CTXEXT>,
    ) -> Result<objectiveai::agent::response::ListAgentResponse, ResponseError> {
        Ok(crate::functions::mock::list_agents())
    }

    async fn list_swarms(
        &self,
        _ctx: &ctx::Context<CTXEXT>,
    ) -> Result<objectiveai::swarm::response::ListSwarmResponse, ResponseError> {
        Ok(crate::functions::mock::list_swarms())
    }

    async fn list_functions(
        &self,
        _ctx: &ctx::Context<CTXEXT>,
    ) -> Result<objectiveai::functions::response::ListFunctionResponse, ResponseError> {
        Ok(crate::functions::mock::list_functions())
    }

    async fn list_profiles(
        &self,
        _ctx: &ctx::Context<CTXEXT>,
    ) -> Result<objectiveai::functions::profiles::response::ListProfileResponse, ResponseError> {
        Ok(crate::functions::mock::list_profiles())
    }

    async fn list_function_profile_pairs(
        &self,
        _ctx: &ctx::Context<CTXEXT>,
    ) -> Result<objectiveai::functions::response::ListFunctionProfilePairResponse, ResponseError>
    {
        unimplemented!("Mock does not support listing function-profile pairs")
    }
}
