use crate::ctx;
use std::sync::Arc;

/// Handler for recording usage after Function invention.
#[async_trait::async_trait]
pub trait UsageHandler<CTXEXT> {
    /// Records usage from a completed Function invention.
    async fn handle_usage(
        &self,
        ctx: ctx::Context<CTXEXT>,
        request: Arc<objectiveai::functions::inventions::request::FunctionInventionCreateParams>,
        response: objectiveai::functions::inventions::response::unary::FunctionInvention,
    );
}
