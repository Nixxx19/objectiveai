//! Router that dispatches to GitHub or Filesystem fetchers based on Remote.

use crate::ctx;
use std::sync::Arc;

/// Routes Function fetch requests to the appropriate sub-fetcher based on [`Remote`].
///
/// [`Remote`]: objectiveai::functions::Remote
pub struct FetcherRouter<G, F, M> {
    /// GitHub sub-fetcher.
    pub github: Arc<G>,
    /// Filesystem sub-fetcher.
    pub filesystem: Arc<F>,
    /// Mock sub-fetcher.
    pub mock: Arc<M>,
}

impl<G, F, M> FetcherRouter<G, F, M> {
    /// Creates a new FetcherRouter with GitHub, Filesystem, and Mock sub-fetchers.
    pub fn new(github: Arc<G>, filesystem: Arc<F>, mock: Arc<M>) -> Self {
        Self { github, filesystem, mock }
    }
}

impl<G, F, M> FetcherRouter<G, F, M> {
    /// Dispatches a Function fetch to the appropriate sub-fetcher based on the remote.
    ///
    /// Alpha function types are transpiled to standard function types before returning.
    pub async fn fetch<CTXEXT>(
        &self,
        ctx: ctx::Context<CTXEXT>,
        remote: objectiveai::functions::Remote,
        owner: &str,
        repository: &str,
        commit: Option<&str>,
    ) -> Result<
        Option<objectiveai::functions::response::GetFunction>,
        objectiveai::error::ResponseError,
    >
    where
        CTXEXT: Send + Sync + 'static,
        G: super::Fetcher<CTXEXT> + Send + Sync + 'static,
        F: super::Fetcher<CTXEXT> + Send + Sync + 'static,
        M: super::Fetcher<CTXEXT> + Send + Sync + 'static,
    {
        let full = match remote {
            objectiveai::functions::Remote::Github => {
                self.github.fetch(ctx, owner, repository, commit).await?
            }
            objectiveai::functions::Remote::Filesystem => {
                self.filesystem
                    .fetch(ctx, owner, repository, commit)
                    .await?
            }
            objectiveai::functions::Remote::Mock => {
                self.mock.fetch(ctx, owner, repository, commit).await?
            }
        };
        Ok(full.map(|f| f.transpile()))
    }
}
