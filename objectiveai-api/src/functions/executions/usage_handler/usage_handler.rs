//! Trait for handling Function execution usage.

use crate::ctx;
use std::sync::Arc;

/// Handler for recording usage after Function execution.
#[async_trait::async_trait]
pub trait UsageHandler<CTXEXT> {
    /// Records usage from a completed Function execution.
    async fn handle_usage<PC: crate::ctx::persistent_cache::PersistentCacheClient>(
        &self,
        ctx: ctx::Context<CTXEXT, PC>,
        request: Arc<objectiveai::functions::executions::request::FunctionExecutionCreateParams>,
        response: objectiveai::functions::executions::response::unary::FunctionExecution,
    );
}
