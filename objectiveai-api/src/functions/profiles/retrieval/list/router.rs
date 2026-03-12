//! Router for listing Profiles across multiple sources.

use crate::ctx;
use std::sync::Arc;

/// Routes Profile listing requests to the appropriate source client.
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

    /// Lists Profiles, optionally filtered by source.
    pub async fn list_profiles<CTXEXT>(
        &self,
        ctx: ctx::Context<CTXEXT>,
        source: Option<objectiveai::functions::profiles::request::ListProfilesSource>,
    ) -> Result<
        objectiveai::functions::profiles::response::ListProfile,
        objectiveai::error::ResponseError,
    >
    where
        CTXEXT: Send + Sync + 'static,
        O: super::Client<CTXEXT> + Send + Sync + 'static,
        F: super::Client<CTXEXT> + Send + Sync + 'static,
        M: super::Client<CTXEXT> + Send + Sync + 'static,
    {
        use objectiveai::functions::profiles::request::ListProfilesSource;

        match source {
            Some(ListProfilesSource::Objectiveai) => {
                self.objectiveai.list_profiles(ctx).await
            }
            Some(ListProfilesSource::Filesystem) => {
                self.filesystem.list_profiles(ctx).await
            }
            Some(ListProfilesSource::Mock) => {
                self.mock.list_profiles(ctx).await
            }
            Some(ListProfilesSource::All) | None => {
                let ctx_o = ctx.clone();
                let ctx_f = ctx.clone();
                let ctx_m = ctx.clone();

                let (objectiveai_result, filesystem_result, mock_result) =
                    futures::future::join3(
                        self.objectiveai.list_profiles(ctx_o),
                        self.filesystem.list_profiles(ctx_f),
                        self.mock.list_profiles(ctx_m),
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

                Ok(objectiveai::functions::profiles::response::ListProfile { data })
            }
        }
    }
}
