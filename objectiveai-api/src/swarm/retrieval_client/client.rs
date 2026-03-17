//! Swarm retrieval client trait definition.

use crate::ctx;

/// Trait for listing swarms and retrieving usage statistics.
#[async_trait::async_trait]
pub trait Client<CTXEXT> {
    /// Lists all swarms.
    async fn list(
        &self,
        ctx: ctx::Context<CTXEXT>,
    ) -> Result<
        objectiveai::swarm::response::ListSwarm,
        objectiveai::error::ResponseError,
    >;

    /// Retrieves usage statistics for an swarm by ID.
    async fn get_usage(
        &self,
        ctx: ctx::Context<CTXEXT>,
        id: &str,
    ) -> Result<
        objectiveai::swarm::response::UsageSwarm,
        objectiveai::error::ResponseError,
    >;
}
