//! ListSource trait — implemented by Mock, Filesystem, and ObjectiveAI.

use crate::ctx;
use objectiveai::error::ResponseError;

/// A source that can list available resources.
///
/// Implemented by Mock, Filesystem, and ObjectiveAI.
/// GitHub does NOT implement this (it has no list endpoint).
#[async_trait::async_trait]
pub trait Client<CTXEXT>: Send + Sync + 'static {
    async fn list_agents(
        &self,
        ctx: &ctx::Context<CTXEXT>,
    ) -> Result<objectiveai::agent::response::ListAgentResponse, ResponseError>;

    async fn list_swarms(
        &self,
        ctx: &ctx::Context<CTXEXT>,
    ) -> Result<objectiveai::swarm::response::ListSwarmResponse, ResponseError>;

    async fn list_functions(
        &self,
        ctx: &ctx::Context<CTXEXT>,
    ) -> Result<objectiveai::functions::response::ListFunctionResponse, ResponseError>;

    async fn list_profiles(
        &self,
        ctx: &ctx::Context<CTXEXT>,
    ) -> Result<objectiveai::functions::profiles::response::ListProfileResponse, ResponseError>;

    /// Only ObjectiveAI implements meaningfully; Mock/Filesystem → `unimplemented!()`
    async fn list_function_profile_pairs(
        &self,
        ctx: &ctx::Context<CTXEXT>,
    ) -> Result<objectiveai::functions::response::ListFunctionProfilePairResponse, ResponseError>;
}
