//! Trait for listing Function-Profile pairs from a single source.

use crate::ctx;

/// Client for listing Function-Profile pairs from a specific source.
#[async_trait::async_trait]
pub trait Client<CTXEXT> {
    /// Lists Function-Profile pairs from this source.
    async fn list_function_profile_pairs(
        &self,
        ctx: ctx::Context<CTXEXT>,
    ) -> Result<
        objectiveai::functions::response::ListFunctionProfilePair,
        objectiveai::error::ResponseError,
    >;
}
