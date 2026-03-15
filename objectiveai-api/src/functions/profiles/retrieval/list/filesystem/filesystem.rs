//! Filesystem implementation of the Profile list client.

use crate::ctx;
use std::sync::Arc;

/// Lists Profiles from the local filesystem.
pub struct FileSystemClient {
    /// The filesystem client.
    pub client: Arc<crate::filesystem::Client>,
}

impl FileSystemClient {
    /// Creates a new filesystem Profile list client.
    pub fn new(client: Arc<crate::filesystem::Client>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl<CTXEXT> super::super::Client<CTXEXT> for FileSystemClient
where
    CTXEXT: Send + Sync + 'static,
{
    async fn list_profiles(
        &self,
        _ctx: ctx::Context<CTXEXT>,
    ) -> Result<
        objectiveai::functions::profiles::response::ListProfile,
        objectiveai::error::ResponseError,
    > {
        Ok(objectiveai::functions::profiles::response::ListProfile {
            data: vec![],
        })
    }
}
