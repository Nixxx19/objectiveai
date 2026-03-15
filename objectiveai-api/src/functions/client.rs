//! Functions client implementation.

use crate::ctx;
use std::sync::Arc;

/// Client for function operations.
pub struct Client<CTXEXT, G, F, M, RTRVL> {
    /// Router for Function definition fetching.
    pub function_fetcher: Arc<super::function_fetcher::FetcherRouter<G, F, M>>,
    /// Client for listing functions and getting usage statistics.
    pub retrieval_client: Arc<RTRVL>,
    pub _ctx_ext: std::marker::PhantomData<CTXEXT>,
}

impl<CTXEXT, G, F, M, RTRVL> Client<CTXEXT, G, F, M, RTRVL> {
    /// Creates a new functions client.
    pub fn new(
        function_fetcher: Arc<super::function_fetcher::FetcherRouter<G, F, M>>,
        retrieval_client: Arc<RTRVL>,
    ) -> Self {
        Self {
            function_fetcher,
            retrieval_client,
            _ctx_ext: std::marker::PhantomData,
        }
    }
}

impl<CTXEXT, G, F, M, RTRVL> Client<CTXEXT, G, F, M, RTRVL>
where
    CTXEXT: Send + Sync + 'static,
    G: super::function_fetcher::Fetcher<CTXEXT> + Send + Sync + 'static,
    F: super::function_fetcher::Fetcher<CTXEXT> + Send + Sync + 'static,
    M: super::function_fetcher::Fetcher<CTXEXT> + Send + Sync + 'static,
    RTRVL: super::retrieval_client::Client<CTXEXT> + Send + Sync + 'static,
{
    /// Lists functions.
    pub async fn list_functions(
        &self,
        ctx: ctx::Context<CTXEXT>,
    ) -> Result<
        objectiveai::functions::response::ListFunction,
        objectiveai::error::ResponseError,
    > {
        self.retrieval_client.list_functions(ctx).await
    }

    /// Retrieves a function by remote/owner/repository/commit.
    pub async fn get_function(
        &self,
        ctx: ctx::Context<CTXEXT>,
        remote: objectiveai::functions::Remote,
        owner: &str,
        repository: &str,
        commit: Option<&str>,
    ) -> Result<
        objectiveai::functions::response::GetFunction,
        objectiveai::error::ResponseError,
    > {
        self.function_fetcher
            .fetch(ctx, remote, owner, repository, commit)
            .await?
            .ok_or_else(|| objectiveai::error::ResponseError {
                code: 404,
                message: serde_json::json!({
                    "kind": "functions",
                    "error": "Function not found"
                }),
            })
    }

    /// Retrieves usage statistics for a function.
    pub async fn get_function_usage(
        &self,
        ctx: ctx::Context<CTXEXT>,
        remote: objectiveai::functions::Remote,
        owner: &str,
        repository: &str,
        commit: Option<&str>,
    ) -> Result<
        objectiveai::functions::response::UsageFunction,
        objectiveai::error::ResponseError,
    > {
        self.retrieval_client
            .get_function_usage(ctx, remote, owner, repository, commit)
            .await
    }
}
