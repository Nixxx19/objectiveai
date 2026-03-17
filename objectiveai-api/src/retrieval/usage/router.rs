//! Usage router — delegates to the ObjectiveAI usage client.
//!
//! No caching needed for usage — these are simple pass-through requests.

use crate::ctx;
use objectiveai::error::ResponseError;
use std::sync::Arc;

/// Routes usage requests to the ObjectiveAI usage client.
///
/// Only ObjectiveAI tracks usage, so there's only one delegate.
pub struct Router<O> {
    pub objectiveai: Arc<O>,
}

impl<O> Router<O> {
    pub fn new(objectiveai: Arc<O>) -> Self {
        Self { objectiveai }
    }
}

impl<O, CTXEXT> Router<O>
where
    O: super::Client<CTXEXT>,
    CTXEXT: Send + Sync + 'static,
{
    pub async fn get_agent_usage(
        &self,
        ctx: &ctx::Context<CTXEXT>,
        params: &objectiveai::agent::request::GetAgentRequest,
    ) -> Result<objectiveai::agent::response::UsageAgentResponse, ResponseError> {
        self.objectiveai.get_agent_usage(ctx, params).await
    }

    pub async fn get_swarm_usage(
        &self,
        ctx: &ctx::Context<CTXEXT>,
        params: &objectiveai::swarm::request::GetSwarmRequest,
    ) -> Result<objectiveai::swarm::response::UsageSwarmResponse, ResponseError> {
        self.objectiveai.get_swarm_usage(ctx, params).await
    }

    pub async fn get_function_usage(
        &self,
        ctx: &ctx::Context<CTXEXT>,
        params: &objectiveai::functions::request::GetFunctionRequest,
    ) -> Result<objectiveai::functions::response::UsageFunctionResponse, ResponseError> {
        self.objectiveai.get_function_usage(ctx, params).await
    }

    pub async fn get_profile_usage(
        &self,
        ctx: &ctx::Context<CTXEXT>,
        params: &objectiveai::functions::profiles::request::GetProfileRequest,
    ) -> Result<objectiveai::functions::profiles::response::UsageProfileResponse, ResponseError> {
        self.objectiveai.get_profile_usage(ctx, params).await
    }

    pub async fn get_function_profile_pair_usage(
        &self,
        ctx: &ctx::Context<CTXEXT>,
        params: &objectiveai::functions::request::GetFunctionProfilePairUsageRequest,
    ) -> Result<objectiveai::functions::response::UsageFunctionProfilePairResponse, ResponseError>
    {
        self.objectiveai.get_function_profile_pair_usage(ctx, params).await
    }
}
