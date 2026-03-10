//! Local filesystem Function fetcher.

use crate::ctx;
use std::sync::Arc;

/// Fetches Functions from the local filesystem.
pub struct FilesystemFetcher {
    pub client: Arc<crate::filesystem::Client>,
}

impl FilesystemFetcher {
    pub fn new(client: Arc<crate::filesystem::Client>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl<CTXEXT> super::super::Fetcher<CTXEXT> for FilesystemFetcher
where
    CTXEXT: Send + Sync + 'static,
{
    async fn fetch(
        &self,
        _ctx: ctx::Context<CTXEXT>,
        owner: &str,
        repository: &str,
        commit: Option<&str>,
    ) -> Result<
        Option<super::super::FullGetFunction>,
        objectiveai::error::ResponseError,
    > {
        match self
            .client
            .read_json::<objectiveai::functions::FullRemoteFunction>(
                crate::filesystem::Kind::Functions,
                owner,
                repository,
                commit,
                "function.json",
            )
            .await
        {
            Ok(Some((function, resolved_commit))) => {
                Ok(Some(super::super::FullGetFunction {
                    remote: objectiveai::functions::Remote::Filesystem,
                    owner: owner.to_string(),
                    repository: repository.to_string(),
                    commit: resolved_commit,
                    inner: function,
                }))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(objectiveai::error::ResponseError::from(&e)),
        }
    }
}
