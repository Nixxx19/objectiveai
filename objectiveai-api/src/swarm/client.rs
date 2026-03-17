//! Swarm client for listing, retrieving, and fetching swarms.

use crate::ctx;
use std::sync::Arc;

/// Client for swarm operations.
///
/// Combines a caching fetcher for swarm definitions with a retrieval
/// client for listing and usage statistics.
pub struct Client<CTXEXT, FENS, RTRVL> {
    /// Caching fetcher for swarm definitions.
    pub swarm_fetcher: Arc<super::fetcher::CachingFetcher<CTXEXT, FENS>>,
    /// Client for listing swarms and getting usage.
    pub retrieval_client: Arc<RTRVL>,
    /// Phantom data for the context extension type.
    pub _ctx_ext: std::marker::PhantomData<CTXEXT>,
}

impl<CTXEXT, FENS, RTRVL> Client<CTXEXT, FENS, RTRVL> {
    /// Creates a new swarm client.
    pub fn new(
        swarm_fetcher: Arc<
            super::fetcher::CachingFetcher<CTXEXT, FENS>,
        >,
        retrieval_client: Arc<RTRVL>,
    ) -> Self {
        Self {
            swarm_fetcher,
            retrieval_client,
            _ctx_ext: std::marker::PhantomData,
        }
    }
}

impl<CTXEXT, FENS, RTRVL> Client<CTXEXT, FENS, RTRVL>
where
    CTXEXT: Send + Sync + 'static,
    FENS: super::fetcher::Fetcher<CTXEXT>
        + Send
        + Sync
        + 'static,
    RTRVL: super::retrieval_client::Client<CTXEXT> + Send + Sync + 'static,
{
    /// Lists all swarms.
    pub async fn list(
        &self,
        ctx: ctx::Context<CTXEXT>,
    ) -> Result<
        objectiveai::swarm::response::ListSwarm,
        objectiveai::error::ResponseError,
    > {
        self.retrieval_client.list(ctx).await
    }

    /// Retrieves an swarm by its ID.
    ///
    /// Returns a 404 error if the swarm is not found.
    pub async fn get(
        &self,
        ctx: ctx::Context<CTXEXT>,
        id: &str,
    ) -> Result<
        objectiveai::swarm::response::GetSwarm,
        objectiveai::error::ResponseError,
    > {
        self.swarm_fetcher
            .fetch(ctx, id)
            .await?
            .ok_or_else(|| objectiveai::error::ResponseError {
                code: 404,
                message: serde_json::json!({
                    "kind": "swarm",
                    "error": "Swarm not found"
                }),
            })
            .map(|(inner, created)| {
                objectiveai::swarm::response::GetSwarm { created, inner }
            })
    }

    /// Retrieves usage statistics for an swarm.
    pub async fn get_usage(
        &self,
        ctx: ctx::Context<CTXEXT>,
        id: &str,
    ) -> Result<
        objectiveai::swarm::response::UsageSwarm,
        objectiveai::error::ResponseError,
    > {
        self.retrieval_client.get_usage(ctx, id).await
    }
}
