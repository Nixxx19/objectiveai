//! Filesystem implementation of the Function list client.

use crate::ctx;
use std::sync::Arc;

/// Lists Functions from the local filesystem.
pub struct FileSystemClient {
    /// The filesystem client.
    pub client: Arc<crate::filesystem::Client>,
}

impl FileSystemClient {
    /// Creates a new filesystem Function list client.
    pub fn new(client: Arc<crate::filesystem::Client>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl<CTXEXT> super::super::Client<CTXEXT> for FileSystemClient
where
    CTXEXT: Send + Sync + 'static,
{
    async fn list_functions(
        &self,
        _ctx: ctx::Context<CTXEXT>,
    ) -> Result<
        objectiveai::functions::response::ListFunction,
        objectiveai::error::ResponseError,
    > {
        Ok(objectiveai::functions::response::ListFunction {
            data: vec![],
        })
    }
}
