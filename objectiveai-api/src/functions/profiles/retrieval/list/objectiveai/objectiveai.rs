//! ObjectiveAI API implementation of the Profile list client.

use crate::ctx;
use std::sync::Arc;

/// Lists Profiles via the ObjectiveAI API.
pub struct ObjectiveAiClient {
    /// The HTTP client for API requests.
    pub client: Arc<crate::objectiveai_http::Client>,
}

impl ObjectiveAiClient {
    /// Creates a new ObjectiveAI Profile list client.
    pub fn new(client: Arc<crate::objectiveai_http::Client>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl<CTXEXT> super::super::Client<CTXEXT> for ObjectiveAiClient
where
    CTXEXT: Send + Sync + 'static + ctx::ContextExt,
{
    async fn list_profiles(
        &self,
        ctx: ctx::Context<CTXEXT>,
    ) -> Result<
        objectiveai::functions::profiles::response::ListProfile,
        objectiveai::error::ResponseError,
    > {
        let client = self.client.with_authorization(&ctx).await;
        objectiveai::functions::profiles::list_profiles(
                &client,
                Some(objectiveai::functions::profiles::request::ListProfilesSource::Objectiveai),
            )
            .await
            .map_err(|e| objectiveai::error::ResponseError::from(&e))
    }
}
