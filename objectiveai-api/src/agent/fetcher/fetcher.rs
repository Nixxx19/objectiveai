//! Agent fetcher trait definition.

use crate::ctx;

/// Trait for fetching Agent definitions by ID.
#[async_trait::async_trait]
pub trait Fetcher<CTXEXT> {
    /// Fetches an Agent by its ID.
    ///
    /// Returns `Ok(None)` if the Agent is not found.
    /// Returns the Agent and its creation timestamp if found.
    async fn fetch(
        &self,
        ctx: ctx::Context<CTXEXT>,
        id: &str,
    ) -> Result<
        Option<(objectiveai::agent::Agent, u64)>,
        objectiveai::error::ResponseError,
    >;
}
