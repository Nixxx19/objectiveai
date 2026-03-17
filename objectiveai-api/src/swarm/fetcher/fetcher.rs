//! Swarm fetcher trait definition.

use crate::ctx;

/// Trait for fetching swarm definitions by ID.
#[async_trait::async_trait]
pub trait Fetcher<CTXEXT> {
    /// Fetches an swarm by its ID.
    ///
    /// Returns `Ok(None)` if the swarm is not found.
    /// Returns the swarm and its creation timestamp if found.
    async fn fetch(
        &self,
        ctx: ctx::Context<CTXEXT>,
        id: &str,
    ) -> Result<
        Option<(objectiveai::swarm::Swarm, u64)>,
        objectiveai::error::ResponseError,
    >;
}
