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
        model: &str,
        models: Option<&[&str]>,
        messages: &[objectiveai::chat::completions::request::Message],
        tools: Option<&[objectiveai::chat::completions::request::Tool]>,
        responses: &[objectiveai::chat::completions::request::RichContent],
    ) -> Result<
        Option<objectiveai::vector::completions::response::Vote>,
        objectiveai::error::ResponseError,
    > {
        let request = objectiveai::vector::completions::cache::request::CacheVoteRequest {
            model: objectiveai::chat::completions::request::Model::Id(
                model.to_string(),
            ),
            models: models.map(|models| {
                models
                    .iter()
                    .map(|model| {
                        objectiveai::chat::completions::request::Model::Id(
                            model.to_string(),
                        )
                    })
                    .collect()
            }),
            messages: messages.to_vec(),
            tools: tools.map(|tools| tools.to_vec()),
            responses: responses.to_vec(),
        };
        match objectiveai::vector::completions::cache::get_cache_vote(
            &self.client,
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
