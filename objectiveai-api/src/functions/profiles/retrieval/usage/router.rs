//! Router for Profile usage retrieval.

use crate::ctx;
use std::sync::Arc;

/// Routes Profile usage requests to the appropriate client.
pub struct UsageRouter<O> {
    /// ObjectiveAI API usage client.
    pub objectiveai: Arc<O>,
}

impl<O> UsageRouter<O> {
    /// Creates a new usage router.
    pub fn new(objectiveai: Arc<O>) -> Self {
        Self { objectiveai }
    }

    /// Retrieves usage statistics for a Profile.
    pub async fn get_profile_usage<CTXEXT>(
        &self,
        ctx: ctx::Context<CTXEXT>,
        remote: objectiveai::functions::Remote,
        owner: &str,
        repository: &str,
        commit: Option<&str>,
    ) -> Result<
        objectiveai::functions::profiles::response::UsageProfile,
        objectiveai::error::ResponseError,
    >
    where
        CTXEXT: Send + Sync + 'static,
        O: super::Client<CTXEXT>,
    {
        self.objectiveai
            .get_profile_usage(ctx, remote, owner, repository, commit)
            .await
    }
}
