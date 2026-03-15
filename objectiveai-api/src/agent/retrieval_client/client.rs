//! Agent retrieval client trait definition.

use crate::ctx;

/// Trait for listing Agents and retrieving usage statistics.
#[async_trait::async_trait]
pub trait Client<CTXEXT> {
    /// Lists all Agents.
    async fn list(
        &self,
        ctx: ctx::Context<CTXEXT>,
    ) -> Result<
        objectiveai::agent::response::ListAgent,
        objectiveai::error::ResponseError,
    >;

    /// Retrieves usage statistics for an Agent by ID.
    async fn get_usage(
        &self,
        ctx: ctx::Context<CTXEXT>,
        id: &str,
    ) -> Result<
        objectiveai::agent::response::UsageAgent,
        objectiveai::error::ResponseError,
    >;
}
