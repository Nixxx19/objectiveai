use crate::ctx;
use std::sync::Arc;
pub struct LogUsageHandler;

#[async_trait::async_trait]
impl<CTXEXT> super::UsageHandler<CTXEXT> for LogUsageHandler
where
    CTXEXT: Send + Sync + 'static,
{
    async fn handle_usage(
        &self,
        _ctx: ctx::Context<CTXEXT>,
        _request: Option<Arc<objectiveai::chat::completions::request::ChatCompletionCreateParams>>,
        response: objectiveai::chat::completions::response::unary::ChatCompletion,
    ) {
        println!(
            "[{}] cost: {}",
            response.id.as_str(),
            response.usage.total_cost,
        );
    }
}
