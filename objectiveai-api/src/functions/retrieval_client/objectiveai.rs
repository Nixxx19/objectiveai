//! ObjectiveAI API implementation of the retrieval client.

use crate::ctx;
use std::sync::Arc;

/// Lists Functions and retrieves usage via the ObjectiveAI API.
pub struct ObjectiveAiClient {
    /// The HTTP client for API requests.
    pub client: Arc<crate::objectiveai_http::Client>,
}

impl ObjectiveAiClient {
    /// Creates a new ObjectiveAI retrieval client.
    pub fn new(client: Arc<crate::objectiveai_http::Client>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl<CTXEXT> super::Client<CTXEXT> for ObjectiveAiClient
where
    CTXEXT: Send + Sync + 'static + ctx::ContextExt,
{
    async fn list_functions(
        &self,
        ctx: ctx::Context<CTXEXT>,
    ) -> Result<
        objectiveai::functions::response::ListFunction,
        objectiveai::error::ResponseError,
    > {
        let client = self.client.with_authorization(&ctx).await;
        objectiveai::functions::list_functions(
                &client,
                Some(objectiveai::functions::request::ListFunctionsSource::Objectiveai),
            )
            .await
            .map_err(|e| objectiveai::error::ResponseError::from(&e))
    }

    async fn get_function_usage(
        &self,
        ctx: ctx::Context<CTXEXT>,
        remote: objectiveai::functions::Remote,
        owner: &str,
        repository: &str,
        commit: Option<&str>,
    ) -> Result<
        objectiveai::functions::response::UsageFunction,
        objectiveai::error::ResponseError,
    > {
        let client = self.client.with_authorization(&ctx).await;
        objectiveai::functions::get_function_usage(&client, remote, owner, repository, commit)
            .await
            .map_err(|e| objectiveai::error::ResponseError::from(&e))
    }
}
