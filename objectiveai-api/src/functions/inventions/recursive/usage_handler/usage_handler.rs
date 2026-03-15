use crate::ctx;
use std::sync::Arc;

/// Handler for recording usage after recursive Function invention.
#[async_trait::async_trait]
pub trait UsageHandler<CTXEXT> {
    /// Records usage from a completed recursive Function invention.
    async fn handle_usage(
        &self,
        ctx: ctx::Context<CTXEXT>,
        request: Arc<objectiveai::functions::inventions::recursive::request::FunctionInventionRecursiveCreateParams>,
        response: objectiveai::functions::inventions::recursive::response::unary::FunctionInventionRecursive,
    );
}
