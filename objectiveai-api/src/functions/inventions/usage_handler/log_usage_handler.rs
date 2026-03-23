use crate::ctx;
use std::sync::Arc;

/// Usage handler that logs invention costs to stdout.
pub struct LogUsageHandler;

#[async_trait::async_trait]
impl<CTXEXT> super::UsageHandler<CTXEXT> for LogUsageHandler
where
    CTXEXT: Send + Sync + 'static,
{
    async fn handle_usage(
        &self,
        ctx: ctx::Context<CTXEXT>,
        _request: Arc<objectiveai::functions::inventions::request::FunctionInventionCreateParams>,
        response: objectiveai::functions::inventions::response::unary::FunctionInvention,
    ) {
        if !ctx.suppress_output {
            println!(
                "[{}] cost: {}",
                response.id.as_str(),
                response.usage.total_cost,
            );
        }
    }
}
