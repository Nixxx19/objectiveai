//! Router for listing Function-Profile pairs across sources.

use crate::ctx;
use std::sync::Arc;

/// Routes Function-Profile pair listing requests to the appropriate source client.
pub struct ListRouter<O> {
    /// ObjectiveAI API listing client.
    pub objectiveai: Arc<O>,
}

impl<O> ListRouter<O> {
    /// Creates a new pair list router.
    pub fn new(objectiveai: Arc<O>) -> Self {
        Self { objectiveai }
    }

    /// Lists Function-Profile pairs, optionally filtered by source.
    pub async fn list_function_profile_pairs<CTXEXT>(
        &self,
        ctx: ctx::Context<CTXEXT>,
        source: Option<objectiveai::functions::request::ListFunctionProfilePairsSource>,
    ) -> Result<
        objectiveai::functions::response::ListFunctionProfilePair,
        objectiveai::error::ResponseError,
    >
    where
        CTXEXT: Send + Sync + 'static,
        O: super::Client<CTXEXT> + Send + Sync + 'static,
    {
        use objectiveai::functions::request::ListFunctionProfilePairsSource;

        match source {
            Some(ListFunctionProfilePairsSource::Objectiveai) | None => {
                self.objectiveai.list_function_profile_pairs(ctx).await
            }
        }
    }
}
