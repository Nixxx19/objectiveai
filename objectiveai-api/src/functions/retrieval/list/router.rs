//! Router for listing Functions across multiple sources.

use crate::ctx;
use std::sync::Arc;

/// Routes Function listing requests to the appropriate source client.
pub struct ListRouter<O, F, M> {
    /// ObjectiveAI API listing client.
    pub objectiveai: Arc<O>,
    /// Filesystem listing client.
    pub filesystem: Arc<F>,
    /// Mock listing client.
    pub mock: Arc<M>,
}

impl<O, F, M> ListRouter<O, F, M> {
    /// Creates a new list router.
    pub fn new(objectiveai: Arc<O>, filesystem: Arc<F>, mock: Arc<M>) -> Self {
        Self {
            objectiveai,
            filesystem,
            mock,
        }
    }

    /// Lists Functions, optionally filtered by source.
    pub async fn list_functions<CTXEXT>(
        &self,
        ctx: ctx::Context<CTXEXT>,
        source: Option<objectiveai::functions::request::ListFunctionsSource>,
    ) -> Result<
        objectiveai::functions::response::ListFunction,
        objectiveai::error::ResponseError,
    >
    where
        CTXEXT: Send + Sync + 'static,
        O: super::Client<CTXEXT> + Send + Sync + 'static,
        F: super::Client<CTXEXT> + Send + Sync + 'static,
        M: super::Client<CTXEXT> + Send + Sync + 'static,
    {
        use objectiveai::functions::request::ListFunctionsSource;

        match source {
            Some(ListFunctionsSource::Objectiveai) => {
                self.objectiveai.list_functions(ctx).await
            }
            Some(ListFunctionsSource::Filesystem) => {
                self.filesystem.list_functions(ctx).await
            }
            Some(ListFunctionsSource::Mock) => {
                self.mock.list_functions(ctx).await
            }
            Some(ListFunctionsSource::All) | None => {
                let ctx_o = ctx.clone();
                let ctx_f = ctx.clone();
                let ctx_m = ctx.clone();

                let (objectiveai_result, filesystem_result, mock_result) =
                    futures::future::join3(
                        self.objectiveai.list_functions(ctx_o),
                        self.filesystem.list_functions(ctx_f),
                        self.mock.list_functions(ctx_m),
                    )
                    .await;

                let mut data = Vec::new();
                if let Ok(r) = objectiveai_result {
                    data.extend(r.data);
                }
                if let Ok(r) = filesystem_result {
                    data.extend(r.data);
                }
                if let Ok(r) = mock_result {
                    data.extend(r.data);
                }

                Ok(objectiveai::functions::response::ListFunction { data })
            }
        }
    }
}
