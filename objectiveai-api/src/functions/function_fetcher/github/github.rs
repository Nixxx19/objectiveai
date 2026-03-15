//! GitHub API implementation of the Function fetcher.

use crate::ctx;
use std::sync::Arc;

/// Fetches Functions from GitHub directly via the GitHub API.
pub struct GithubFetcher {
    pub client: Arc<crate::github::Client>,
}

impl GithubFetcher {
    pub fn new(client: Arc<crate::github::Client>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl<CTXEXT> super::super::Fetcher<CTXEXT> for GithubFetcher
where
    CTXEXT: Send + Sync + 'static,
{
    async fn fetch(
        &self,
        ctx: ctx::Context<CTXEXT>,
        owner: &str,
        repository: &str,
        commit: Option<&str>,
    ) -> Result<
        Option<super::super::FullGetFunction>,
        objectiveai::error::ResponseError,
    > {
        self.client
            .clone()
            .fetch_function(ctx, owner, repository, commit)
            .await
    }
}
