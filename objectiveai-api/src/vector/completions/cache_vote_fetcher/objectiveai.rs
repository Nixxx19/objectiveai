//! ObjectiveAI API implementation of the cache vote fetcher.

use crate::ctx;
use objectiveai::error::StatusError;
use std::sync::Arc;

/// Fetches cached votes from the ObjectiveAI API.
pub struct ObjectiveAiFetcher {
    /// The HTTP client for API requests.
    pub client: Arc<crate::objectiveai_http::Client>,
}

impl ObjectiveAiFetcher {
    /// Creates a new ObjectiveAI cache vote fetcher.
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
        agent: &objectiveai::agent::completions::request::Agent,
        agents: Option<&[objectiveai::agent::completions::request::Agent]>,
        messages: &[objectiveai::agent::completions::message::Message],
        responses: &[objectiveai::agent::completions::message::RichContent],
    ) -> Result<
        Option<objectiveai::vector::completions::response::Vote>,
        objectiveai::error::ResponseError,
    > {
        let client = self.client.with_authorization(&ctx).await;
        let request = objectiveai::vector::completions::cache::request::CacheVoteRequest::Ref(
            objectiveai::vector::completions::cache::request::CacheVoteRequestRef {
                agent,
                agents,
                messages,
                responses,
            },
        );
        match objectiveai::vector::completions::cache::get_cache_vote(
            &client,
            &request,
        )
        .await
        {
            Ok(vote) => Ok(vote.vote),
            Err(e) if e.status() == 404 => Ok(None),
            Err(e) => Err(objectiveai::error::ResponseError::from(&e)),
        }
    }
}
