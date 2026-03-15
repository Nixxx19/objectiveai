//! Mock implementation of the Profile list client.

use crate::ctx;

/// Mock Profile list client.
pub struct MockClient;

#[async_trait::async_trait]
impl<CTXEXT> super::super::Client<CTXEXT> for MockClient
where
    CTXEXT: Send + Sync + 'static,
{
    async fn list_profiles(
        &self,
        _ctx: ctx::Context<CTXEXT>,
    ) -> Result<
        objectiveai::functions::profiles::response::ListProfile,
        objectiveai::error::ResponseError,
    > {
        Ok(crate::functions::mock::list_profiles())
    }
}
