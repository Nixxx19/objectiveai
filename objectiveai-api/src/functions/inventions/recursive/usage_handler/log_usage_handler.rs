use crate::ctx;
use std::sync::Arc;

/// Usage handler that logs recursive invention costs to stdout.
pub struct LogUsageHandler;

#[async_trait::async_trait]
impl<CTXEXT> super::UsageHandler<CTXEXT> for LogUsageHandler
where
    CTXEXT: Send + Sync + 'static,
{
    async fn handle_usage(
        &self,
        _ctx: ctx::Context<CTXEXT>,
        _request: Arc<objectiveai::functions::inventions::recursive::request::FunctionInventionRecursiveCreateParams>,
        response: objectiveai::functions::inventions::recursive::response::unary::FunctionInventionRecursive,
    ) {
        println!(
            "[{}] cost: {}",
            response.id.as_str(),
            response.usage.total_cost,
        );
    }
}
