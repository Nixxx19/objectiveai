//! Simple logging usage handler for development.

use crate::ctx;
use std::sync::Arc;

/// Usage handler that logs completion costs to stdout.
pub struct LogUsageHandler;

impl<CTXEXT> super::UsageHandler<CTXEXT> for LogUsageHandler
where
    CTXEXT: Send + Sync + 'static,
{
    fn handle_usage(
        &self,
        _ctx: ctx::Context<CTXEXT>,
        _request: Arc<objectiveai::agent::completions::request::AgentCompletionCreateParams>,
        response: objectiveai::agent::completions::response::unary::AgentCompletion,
    ) -> impl std::future::Future<Output = ()> + Send + 'static {
        async move {
            println!(
                "[{}] cost: {}",
                response.id.as_str(),
                response.usage.total_cost,
            );
        }
    }
}
