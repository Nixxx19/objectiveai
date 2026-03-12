//! Trait for listing Functions from a single source.

use crate::ctx;

/// Client for listing Functions from a specific source.
#[async_trait::async_trait]
pub trait Client<CTXEXT> {
    /// Lists Functions from this source.
    async fn list_functions(
        &self,
        ctx: ctx::Context<CTXEXT>,
    ) -> Result<
        objectiveai::functions::response::ListFunction,
        objectiveai::error::ResponseError,
    >;
}
