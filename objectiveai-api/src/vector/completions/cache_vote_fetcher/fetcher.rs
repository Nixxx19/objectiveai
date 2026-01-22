use crate::ctx;

#[async_trait::async_trait]
pub trait Fetcher<CTXEXT> {
    async fn fetch(
        &self,
        ctx: ctx::Context<CTXEXT>,
        model: &objectiveai::chat::completions::request::Model,
        models: Option<&[objectiveai::chat::completions::request::Model]>,
        messages: &[objectiveai::chat::completions::request::Message],
        tools: Option<&[objectiveai::chat::completions::request::Tool]>,
        responses: &[objectiveai::chat::completions::request::RichContent],
    ) -> Result<
        Option<objectiveai::vector::completions::response::Vote>,
        objectiveai::error::ResponseError,
    >;
}
