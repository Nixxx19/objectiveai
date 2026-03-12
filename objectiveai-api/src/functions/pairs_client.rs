//! Function-Profile pairs client implementation.

use crate::ctx;
use std::sync::Arc;

/// Client for Function-Profile pair operations.
pub struct PairsClient<LO, UO> {
    /// Router for listing Function-Profile pairs from multiple sources.
    pub list_router: Arc<super::pair_retrieval::list::ListRouter<LO>>,
    /// Router for Function-Profile pair usage statistics.
    pub usage_router: Arc<super::pair_retrieval::usage::UsageRouter<UO>>,
}

impl<LO, UO> PairsClient<LO, UO> {
    /// Creates a new pairs client.
    pub fn new(
        list_router: Arc<super::pair_retrieval::list::ListRouter<LO>>,
        usage_router: Arc<super::pair_retrieval::usage::UsageRouter<UO>>,
    ) -> Self {
        Self {
            list_router,
            usage_router,
        }
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
        LO: super::pair_retrieval::list::Client<CTXEXT> + Send + Sync + 'static,
    {
        self.list_router
            .list_function_profile_pairs(ctx, source)
            .await
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
        UO: super::pair_retrieval::usage::Client<CTXEXT>,
    {
        self.usage_router
            .get_function_profile_pair_usage(
                ctx, fremote, fowner, frepository, fcommit, premote, powner, prepository, pcommit,
            )
            .await
    }
}
