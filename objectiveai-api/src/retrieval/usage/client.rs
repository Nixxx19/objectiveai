//! Usage trait — only ObjectiveAI tracks usage statistics.

use crate::ctx;
use objectiveai::error::ResponseError;

/// Retrieves usage statistics for resources.
///
/// Only ObjectiveAI implements this.
#[async_trait::async_trait]
pub trait Client<CTXEXT>: Send + Sync + 'static {
    async fn get_agent_usage<PC: crate::ctx::persistent_cache::PersistentCacheClient>(
        &self,
        ctx: &ctx::Context<CTXEXT, PC>,
        params: &objectiveai::agent::request::GetAgentRequest,
    ) -> Result<objectiveai::agent::response::UsageAgentResponse, ResponseError>;

    async fn get_swarm_usage<PC: crate::ctx::persistent_cache::PersistentCacheClient>(
        &self,
        ctx: &ctx::Context<CTXEXT, PC>,
        params: &objectiveai::swarm::request::GetSwarmRequest,
    ) -> Result<objectiveai::swarm::response::UsageSwarmResponse, ResponseError>;

    async fn get_function_usage<PC: crate::ctx::persistent_cache::PersistentCacheClient>(
        &self,
        ctx: &ctx::Context<CTXEXT, PC>,
        params: &objectiveai::functions::request::GetFunctionRequest,
    ) -> Result<objectiveai::functions::response::UsageFunctionResponse, ResponseError>;

    async fn get_profile_usage<PC: crate::ctx::persistent_cache::PersistentCacheClient>(
        &self,
        ctx: &ctx::Context<CTXEXT, PC>,
        params: &objectiveai::functions::profiles::request::GetProfileRequest,
    ) -> Result<objectiveai::functions::profiles::response::UsageProfileResponse, ResponseError>;

    async fn get_prompt_usage<PC: crate::ctx::persistent_cache::PersistentCacheClient>(
        &self,
        ctx: &ctx::Context<CTXEXT, PC>,
        params: &objectiveai::functions::inventions::prompts::request::GetPromptRequest,
    ) -> Result<objectiveai::functions::inventions::prompts::response::UsagePromptResponse, ResponseError>;

    async fn get_function_profile_pair_usage<PC: crate::ctx::persistent_cache::PersistentCacheClient>(
        &self,
        ctx: &ctx::Context<CTXEXT, PC>,
        params: &objectiveai::functions::request::GetFunctionProfilePairUsageRequest,
    ) -> Result<objectiveai::functions::response::UsageFunctionProfilePairResponse, ResponseError>;
}
