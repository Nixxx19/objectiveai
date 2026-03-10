//! ObjectiveAI API implementation of the pair retrieval client.

use crate::ctx;
use std::sync::Arc;

/// Lists Function-Profile pairs and retrieves usage via the ObjectiveAI API.
pub struct ObjectiveAiClient {
    /// The HTTP client for API requests.
    pub client: Arc<crate::objectiveai_http::Client>,
}

impl ObjectiveAiClient {
    /// Creates a new ObjectiveAI pair retrieval client.
    pub fn new(client: Arc<crate::objectiveai_http::Client>) -> Self {
        Self { client }
    }
}

#[async_trait::async_trait]
impl<CTXEXT> super::Client<CTXEXT> for ObjectiveAiClient
where
    CTXEXT: Send + Sync + 'static + ctx::ContextExt,
{
    async fn list_function_profile_pairs(
        &self,
        ctx: ctx::Context<CTXEXT>,
    ) -> Result<
        objectiveai::functions::response::ListFunctionProfilePair,
        objectiveai::error::ResponseError,
    > {
        let client = self.client.with_authorization(&ctx).await;
        objectiveai::functions::list_function_profile_pairs(&client)
            .await
            .map_err(|e| objectiveai::error::ResponseError::from(&e))
    }

    async fn get_function_profile_pair(
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
        objectiveai::functions::response::GetFunctionProfilePair,
        objectiveai::error::ResponseError,
    > {
        let client = self.client.with_authorization(&ctx).await;
        objectiveai::functions::get_function_profile_pair(
            &client,
            fremote,
            fowner,
            frepository,
            fcommit,
            premote,
            powner,
            prepository,
            pcommit,
        )
        .await
        .map_err(|e| objectiveai::error::ResponseError::from(&e))
    }

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
    > {
        let client = self.client.with_authorization(&ctx).await;
        objectiveai::functions::get_function_profile_pair_usage(
            &client,
            fremote,
            fowner,
            frepository,
            fcommit,
            premote,
            powner,
            prepository,
            pcommit,
        )
        .await
        .map_err(|e| objectiveai::error::ResponseError::from(&e))
    }
}
