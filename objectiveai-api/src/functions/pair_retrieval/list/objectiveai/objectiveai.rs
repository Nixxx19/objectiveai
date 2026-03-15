//! ObjectiveAI API implementation of the Function-Profile pair list client.

use crate::ctx;
use std::sync::Arc;

/// Lists Function-Profile pairs via the ObjectiveAI API.
pub struct ObjectiveAiClient {
    /// The HTTP client for API requests.
    pub client: Arc<crate::objectiveai_http::Client>,
}

impl ObjectiveAiClient {
    /// Creates a new ObjectiveAI pair list client.
    pub fn new(client: Arc<crate::objectiveai_http::Client>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl<CTXEXT> super::super::Client<CTXEXT> for ObjectiveAiClient
where
    CTXEXT: Send + Sync + 'static + ctx::ContextExt,
{
    async fn list_function_profile_pairs(
        &self,
        ctx: ctx::Context<CTXEXT>,
    ) -> Result<
        objectiveai::functions::response::ListFunctionProfilePair,
        objectiveai::error::ResponseError,
    > {
        let client = self.client.with_authorization(&ctx).await;
        objectiveai::functions::list_function_profile_pairs(
                &client,
                Some(objectiveai::functions::request::ListFunctionProfilePairsSource::Objectiveai),
            )
            .await
            .map_err(|e| objectiveai::error::ResponseError::from(&e))
    }
}
