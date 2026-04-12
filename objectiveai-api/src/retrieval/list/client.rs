//! ListSource trait — implemented by Mock, Filesystem, and ObjectiveAI.

use crate::ctx;
use objectiveai::error::ResponseError;

/// A source that can list available resources.
///
/// Implemented by Mock, Filesystem, and ObjectiveAI.
/// GitHub does NOT implement this (it has no list endpoint).
#[async_trait::async_trait]
pub trait Client<CTXEXT>: Send + Sync + 'static {
    async fn list_agents<PC: crate::ctx::persistent_cache::PersistentCacheClient>(
        &self,
        ctx: &ctx::Context<CTXEXT, PC>,
    ) -> Result<objectiveai::agent::response::ListAgentResponse, ResponseError>;

    async fn list_swarms<PC: crate::ctx::persistent_cache::PersistentCacheClient>(
        &self,
        ctx: &ctx::Context<CTXEXT, PC>,
    ) -> Result<objectiveai::swarm::response::ListSwarmResponse, ResponseError>;

    async fn list_functions<PC: crate::ctx::persistent_cache::PersistentCacheClient>(
        &self,
        ctx: &ctx::Context<CTXEXT, PC>,
    ) -> Result<objectiveai::functions::response::ListFunctionResponse, ResponseError>;

    async fn list_profiles<PC: crate::ctx::persistent_cache::PersistentCacheClient>(
        &self,
        ctx: &ctx::Context<CTXEXT, PC>,
    ) -> Result<objectiveai::functions::profiles::response::ListProfileResponse, ResponseError>;

    async fn list_prompts<PC: crate::ctx::persistent_cache::PersistentCacheClient>(
        &self,
        ctx: &ctx::Context<CTXEXT, PC>,
    ) -> Result<objectiveai::functions::inventions::prompts::response::ListPromptResponse, ResponseError>;

    /// Only ObjectiveAI implements meaningfully; Mock/Filesystem → `unimplemented!()`
    async fn list_function_profile_pairs<PC: crate::ctx::persistent_cache::PersistentCacheClient>(
        &self,
        ctx: &ctx::Context<CTXEXT, PC>,
    ) -> Result<objectiveai::functions::response::ListFunctionProfilePairResponse, ResponseError>;
}
