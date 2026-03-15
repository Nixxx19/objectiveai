//! ObjectiveAI Agent fetcher implementation.

use crate::ctx;
use objectiveai::error::StatusError;
use std::sync::Arc;

/// Fetches Agents from the ObjectiveAI HTTP API.
pub struct ObjectiveAiFetcher {
    /// The underlying HTTP client.
    pub client: Arc<crate::objectiveai_http::Client>,
}

impl ObjectiveAiFetcher {
    /// Creates a new ObjectiveAI Agent fetcher.
    pub fn new(client: Arc<crate::objectiveai_http::Client>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl<CTXEXT> super::Fetcher<CTXEXT> for ObjectiveAiFetcher
where
    CTXEXT: Send + Sync + 'static + ctx::ContextExt,
{
    async fn fetch(
        &self,
        ctx: ctx::Context<CTXEXT>,
        id: &str,
    ) -> Result<
        Option<(objectiveai::agent::Agent, u64)>,
        objectiveai::error::ResponseError,
    > {
        let client = self.client.with_authorization(&ctx).await;
        match objectiveai::agent::get_agent(&client, id).await {
            Ok(agent) => Ok(Some((agent.inner, agent.created))),
            Err(e) if e.status() == 404 => Ok(None),
            Err(e) => Err(objectiveai::error::ResponseError::from(&e)),
        }
    }
}
