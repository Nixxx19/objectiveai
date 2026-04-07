//! Trait for fetching votes from the global cache.

use crate::ctx;

/// Fetches votes from the global ObjectiveAI vote cache.
#[async_trait::async_trait]
pub trait Fetcher<CTXEXT> {
    /// Requests a cached vote matching the given parameters.
    ///
    /// Returns None if no cached vote exists.
    async fn fetch<PC: crate::ctx::persistent_cache::PersistentCacheClient>(
        &self,
        ctx: ctx::Context<CTXEXT, PC>,
        agent: &objectiveai::agent::InlineAgentBaseWithFallbacksOrRemote,
        messages: &[objectiveai::agent::completions::message::Message],
        responses: &[objectiveai::agent::completions::message::RichContent],
    ) -> Result<
        Option<objectiveai::vector::completions::response::Vote>,
        objectiveai::error::ResponseError,
    >;
}
