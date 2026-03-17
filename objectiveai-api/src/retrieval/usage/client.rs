//! Usage trait — only ObjectiveAI tracks usage statistics.

use crate::ctx;
use objectiveai::error::ResponseError;

/// Retrieves usage statistics for resources.
///
/// Only ObjectiveAI implements this.
#[async_trait::async_trait]
pub trait Client<CTXEXT>: Send + Sync + 'static {
    async fn get_agent_usage(
        &self,
        ctx: &ctx::Context<CTXEXT>,
        params: &objectiveai::agent::request::GetAgentRequest,
    ) -> Result<objectiveai::agent::response::UsageAgentResponse, ResponseError>;

    async fn get_swarm_usage(
        &self,
        ctx: &ctx::Context<CTXEXT>,
        params: &objectiveai::swarm::request::GetSwarmRequest,
    ) -> Result<objectiveai::swarm::response::UsageSwarmResponse, ResponseError>;

    async fn get_function_usage(
        &self,
        ctx: &ctx::Context<CTXEXT>,
        params: &objectiveai::functions::request::GetFunctionRequest,
    ) -> Result<objectiveai::functions::response::UsageFunctionResponse, ResponseError>;

    async fn get_profile_usage(
        &self,
        ctx: &ctx::Context<CTXEXT>,
        params: &objectiveai::functions::profiles::request::GetProfileRequest,
    ) -> Result<objectiveai::functions::profiles::response::UsageProfileResponse, ResponseError>;

    async fn get_function_profile_pair_usage(
        &self,
        ctx: &ctx::Context<CTXEXT>,
        params: &objectiveai::functions::request::GetFunctionProfilePairUsageRequest,
    ) -> Result<objectiveai::functions::response::UsageFunctionProfilePairResponse, ResponseError>;
}
