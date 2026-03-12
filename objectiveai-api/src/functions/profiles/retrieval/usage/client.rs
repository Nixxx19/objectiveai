//! Trait for retrieving Profile usage statistics.

use crate::ctx;

/// Client for retrieving Profile usage statistics.
#[async_trait::async_trait]
pub trait Client<CTXEXT> {
    /// Retrieves usage statistics for a Profile.
    async fn get_profile_usage(
        &self,
        ctx: ctx::Context<CTXEXT>,
        remote: objectiveai::functions::Remote,
        owner: &str,
        repository: &str,
        commit: Option<&str>,
    ) -> Result<
        objectiveai::functions::profiles::response::UsageProfile,
        objectiveai::error::ResponseError,
    >;
}
