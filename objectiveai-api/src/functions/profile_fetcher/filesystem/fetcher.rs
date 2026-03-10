//! Local filesystem Profile fetcher.

use crate::ctx;
use std::sync::Arc;

/// Fetches Profiles from the local filesystem.
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
        Option<objectiveai::functions::profiles::response::GetProfile>,
        objectiveai::error::ResponseError,
    > {
        match self
            .client
            .read_json::<objectiveai::functions::RemoteProfile>(
                crate::filesystem::Kind::Profiles,
                owner,
                repository,
                commit,
                "profile.json",
            )
            .await
        {
            Ok(Some((profile, resolved_commit))) => {
                Ok(Some(
                    objectiveai::functions::profiles::response::GetProfile {
                        remote: objectiveai::functions::Remote::Filesystem,
                        owner: owner.to_string(),
                        repository: repository.to_string(),
                        commit: resolved_commit,
                        inner: profile,
                    },
                ))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(objectiveai::error::ResponseError::from(&e)),
        }
    }
}
