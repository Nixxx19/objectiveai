//! Trait for retrieving Function usage statistics.

use crate::ctx;

/// Client for retrieving Function usage statistics.
#[async_trait::async_trait]
pub trait Client<CTXEXT> {
    /// Retrieves usage statistics for a Function.
    async fn get_function_usage(
        &self,
        ctx: ctx::Context<CTXEXT>,
        remote: objectiveai::functions::Remote,
        owner: &str,
        repository: &str,
        commit: Option<&str>,
    ) -> Result<
        objectiveai::functions::response::UsageFunction,
        objectiveai::error::ResponseError,
    >;
}
