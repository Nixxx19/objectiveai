//! Mock implementation of the Function list client.

use crate::ctx;

/// Mock Function list client.
pub struct MockClient;

#[async_trait::async_trait]
impl<CTXEXT> super::super::Client<CTXEXT> for MockClient
where
    CTXEXT: Send + Sync + 'static,
{
    async fn list_functions(
        &self,
        _ctx: ctx::Context<CTXEXT>,
    ) -> Result<
        objectiveai::functions::response::ListFunction,
        objectiveai::error::ResponseError,
    > {
        Ok(crate::functions::mock::list_functions())
    }
}
