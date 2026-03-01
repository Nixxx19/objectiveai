//! ObjectiveAI Agent retrieval client implementation.

use crate::ctx;
use std::sync::Arc;

/// Retrieval client that delegates to the ObjectiveAI HTTP API.
pub struct ObjectiveAiClient {
    /// The underlying HTTP client.
    pub client: Arc<objectiveai::HttpClient>,
}

impl ObjectiveAiClient {
    /// Creates a new ObjectiveAI retrieval client.
    pub fn new(client: Arc<objectiveai::HttpClient>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl<CTXEXT> super::Client<CTXEXT> for ObjectiveAiClient
where
    CTXEXT: Send + Sync + 'static,
{
    async fn list(
        &self,
        _ctx: ctx::Context<CTXEXT>,
    ) -> Result<
        objectiveai::agent::response::ListAgent,
        objectiveai::error::ResponseError,
    > {
        objectiveai::agent::list_agents(&self.client)
            .await
            .map_err(|e| objectiveai::error::ResponseError::from(&e))
    }

    async fn get_usage(
        &self,
        _ctx: ctx::Context<CTXEXT>,
        id: &str,
    ) -> Result<
        objectiveai::agent::response::UsageAgent,
        objectiveai::error::ResponseError,
    > {
        objectiveai::agent::get_agent_usage(&self.client, id)
            .await
            .map_err(|e| objectiveai::error::ResponseError::from(&e))
    }
}
