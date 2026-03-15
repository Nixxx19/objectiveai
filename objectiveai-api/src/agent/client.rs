//! Agent client for listing, retrieving, and fetching Agents.

use crate::ctx;
use std::sync::Arc;

/// Client for Agent operations.
///
/// Combines a caching fetcher for Agent definitions with a retrieval
/// client for listing and usage statistics.
pub struct Client<CTXEXT, FAGENT, RTRVL> {
    /// Caching fetcher for Agent definitions.
    pub agent_fetcher: Arc<super::fetcher::CachingFetcher<CTXEXT, FAGENT>>,
    /// Client for listing Agents and getting usage.
    pub retrieval_client: Arc<RTRVL>,
    /// Phantom data for the context extension type.
    pub _ctx_ext: std::marker::PhantomData<CTXEXT>,
}

impl<CTXEXT, FAGENT, RTRVL> Client<CTXEXT, FAGENT, RTRVL> {
    /// Creates a new Agent client.
    pub fn new(
        agent_fetcher: Arc<
            super::fetcher::CachingFetcher<
                CTXEXT,
                FAGENT,
            >,
        >,
        retrieval_client: Arc<RTRVL>,
    ) -> Self {
        Self {
            agent_fetcher,
            retrieval_client,
            _ctx_ext: std::marker::PhantomData,
        }
    }
}

impl<CTXEXT, FAGENT, RTRVL> Client<CTXEXT, FAGENT, RTRVL>
where
    CTXEXT: Send + Sync + 'static,
    FAGENT: super::fetcher::Fetcher<CTXEXT>
        + Send
        + Sync
        + 'static,
    RTRVL: super::retrieval_client::Client<CTXEXT> + Send + Sync + 'static,
{
    /// Lists all Agents.
    pub async fn list(
        &self,
        ctx: ctx::Context<CTXEXT>,
    ) -> Result<
        objectiveai::agent::response::ListAgent,
        objectiveai::error::ResponseError,
    > {
        self.retrieval_client.list(ctx).await
    }

    /// Retrieves an Agent by its ID.
    ///
    /// Returns a 404 error if the Agent is not found.
    pub async fn get(
        &self,
        ctx: ctx::Context<CTXEXT>,
        id: &str,
    ) -> Result<
        objectiveai::agent::response::GetAgent,
        objectiveai::error::ResponseError,
    > {
        self.agent_fetcher
            .fetch(ctx, id)
            .await?
            .ok_or_else(|| objectiveai::error::ResponseError {
                code: 404,
                message: serde_json::json!({
                    "kind": "agent",
                    "error": "Agent not found"
                }),
            })
            .map(|(inner, created)| {
                objectiveai::agent::response::GetAgent {
                    created,
                    inner,
                }
            })
    }

    /// Retrieves usage statistics for an Agent.
    pub async fn get_usage(
        &self,
        ctx: ctx::Context<CTXEXT>,
        id: &str,
    ) -> Result<
        objectiveai::agent::response::UsageAgent,
        objectiveai::error::ResponseError,
    > {
        self.retrieval_client.get_usage(ctx, id).await
    }
}
