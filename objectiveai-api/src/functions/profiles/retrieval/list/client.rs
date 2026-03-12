//! Trait for listing Profiles from a single source.

use crate::ctx;

/// Client for listing Profiles from a specific source.
#[async_trait::async_trait]
pub trait Client<CTXEXT> {
    /// Lists Profiles from this source.
    async fn list_profiles(
        &self,
        ctx: ctx::Context<CTXEXT>,
    ) -> Result<
        objectiveai::functions::profiles::response::ListProfile,
        objectiveai::error::ResponseError,
    >;
}
