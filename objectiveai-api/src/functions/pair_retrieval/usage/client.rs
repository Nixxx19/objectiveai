//! Trait for retrieving Function-Profile pair usage statistics.

use crate::ctx;

/// Client for retrieving Function-Profile pair usage statistics.
#[async_trait::async_trait]
pub trait Client<CTXEXT> {
    /// Retrieves usage statistics for a Function-Profile pair.
    async fn get_function_profile_pair_usage(
        &self,
        ctx: ctx::Context<CTXEXT>,
        fremote: objectiveai::functions::Remote,
        fowner: &str,
        frepository: &str,
        fcommit: Option<&str>,
        premote: objectiveai::functions::Remote,
        powner: &str,
        prepository: &str,
        pcommit: Option<&str>,
    ) -> Result<
        objectiveai::functions::response::UsageFunctionProfilePair,
        objectiveai::error::ResponseError,
    >;
}
