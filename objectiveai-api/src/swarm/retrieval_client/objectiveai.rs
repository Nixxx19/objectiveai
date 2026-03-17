//! ObjectiveAI swarm retrieval client implementation.

use crate::ctx;
use std::sync::Arc;

/// Retrieval client that delegates to the ObjectiveAI HTTP API.
pub struct ObjectiveAiClient {
    /// The underlying HTTP client.
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
    async fn list(
        &self,
        ctx: ctx::Context<CTXEXT>,
    ) -> Result<
        objectiveai::swarm::response::ListSwarm,
        objectiveai::error::ResponseError,
    > {
        let client = self.client.with_authorization(&ctx).await;
        objectiveai::swarm::list_swarms(&client)
            .await
            .map_err(|e| objectiveai::error::ResponseError::from(&e))
    }

    async fn get_usage(
        &self,
        ctx: ctx::Context<CTXEXT>,
        id: &str,
    ) -> Result<
        objectiveai::swarm::response::UsageSwarm,
        objectiveai::error::ResponseError,
    > {
        let client = self.client.with_authorization(&ctx).await;
        objectiveai::swarm::get_swarm_usage(&client, id)
            .await
            .map_err(|e| objectiveai::error::ResponseError::from(&e))
    }
}
