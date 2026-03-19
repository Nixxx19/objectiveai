//! ObjectiveAI API implementation of the Profile computation client.

use crate::ctx;
use futures::{Stream, TryStreamExt};
use std::sync::Arc;

/// Computes Profiles via the ObjectiveAI API.
pub struct ObjectiveAiClient {
    /// The HTTP client for API requests.
    pub client: Arc<crate::objectiveai_http::Client>,
}

impl ObjectiveAiClient {
    /// Creates a new ObjectiveAI Profile computation client.
    pub fn new(client: Arc<crate::objectiveai_http::Client>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl<CTXEXT> super::Client<CTXEXT> for ObjectiveAiClient
where
    CTXEXT: Send + Sync + 'static + ctx::ContextExt,
{
    async fn create_unary(
        &self,
        ctx: ctx::Context<CTXEXT>,
        request: Arc<
            objectiveai::functions::profiles::computations::request::FunctionProfileComputationCreateParams,
        >,
    ) -> Result<
        objectiveai::functions::profiles::computations::response::unary::FunctionProfileComputation,
        objectiveai::error::ResponseError,
    >{
        let client = self.client.with_authorization(&ctx).await;
        objectiveai::functions::profiles::computations::compute_profile_unary(
            &client,
            (*request).clone(),
        )
        .await
        .map_err(|e| objectiveai::error::ResponseError::from(&e))
    }

    async fn create_streaming(
        &self,
        ctx: ctx::Context<CTXEXT>,
        request: Arc<
            objectiveai::functions::profiles::computations::request::FunctionProfileComputationCreateParams,
        >,
    ) -> Result<
        impl Stream<Item = Result<
            objectiveai::functions::profiles::computations::response::streaming::FunctionProfileComputationChunk,
            objectiveai::error::ResponseError,
        >>
            + Send
            + 'static,
        objectiveai::error::ResponseError,
    >{
        let client = self.client.with_authorization(&ctx).await;
        let stream = objectiveai::functions::profiles::computations::compute_profile_streaming(
            &client,
            (*request).clone(),
        )
        .await
        .map_err(|e| objectiveai::error::ResponseError::from(&e))?;
        Ok(stream.map_err(|e| objectiveai::error::ResponseError::from(&e)))
    }
}
