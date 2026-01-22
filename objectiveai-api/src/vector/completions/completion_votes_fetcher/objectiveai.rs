use crate::ctx;
use objectiveai::error::StatusError;
use std::sync::Arc;

pub struct ObjectiveAiFetcher {
    pub client: Arc<objectiveai::HttpClient>,
}

impl ObjectiveAiFetcher {
    pub fn new(client: Arc<objectiveai::HttpClient>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl<CTXEXT> super::Fetcher<CTXEXT> for ObjectiveAiFetcher
where
    CTXEXT: Send + Sync + 'static,
{
    async fn fetch(
        &self,
        _ctx: ctx::Context<CTXEXT>,
        id: &str,
    ) -> Result<
        Option<Vec<objectiveai::vector::completions::response::Vote>>,
        objectiveai::error::ResponseError,
    > {
        match objectiveai::vector::completions::cache::get_completion_votes(
            &self.client,
            id,
        )
        .await
        {
            Ok(votes) => Ok(votes.data),
            Err(e) if e.status() == 404 => Ok(None),
            Err(e) => Err(objectiveai::error::ResponseError::from(&e)),
        }
    }
}
