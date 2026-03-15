//! Router for Function-Profile pair usage retrieval.

use crate::ctx;
use std::sync::Arc;

/// Routes Function-Profile pair usage requests to the appropriate client.
pub struct UsageRouter<O> {
    /// ObjectiveAI API usage client.
    pub objectiveai: Arc<O>,
}

impl<O> UsageRouter<O> {
    /// Creates a new usage router.
    pub fn new(objectiveai: Arc<O>) -> Self {
        Self { objectiveai }
    }

    /// Retrieves usage statistics for a Function-Profile pair.
    pub async fn get_function_profile_pair_usage<CTXEXT>(
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
    >
    where
        CTXEXT: Send + Sync + 'static,
        O: super::Client<CTXEXT>,
    {
        self.objectiveai
            .get_function_profile_pair_usage(
                ctx, fremote, fowner, frepository, fcommit, premote, powner, prepository, pcommit,
            )
            .await
    }
}
