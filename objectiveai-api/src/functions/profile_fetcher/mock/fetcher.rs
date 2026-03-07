//! Mock Profile fetcher that always returns None.

use crate::ctx;

/// Mock Profile fetcher for testing.
pub struct MockFetcher;

#[async_trait::async_trait]
impl<CTXEXT> super::super::Fetcher<CTXEXT> for MockFetcher
where
    CTXEXT: Send + Sync + 'static,
{
    async fn fetch(
        &self,
        _ctx: ctx::Context<CTXEXT>,
        _owner: &str,
        _repository: &str,
        _commit: Option<&str>,
    ) -> Result<
        Option<objectiveai::functions::profiles::response::GetProfile>,
        objectiveai::error::ResponseError,
    > {
        Ok(None)
    }
}
