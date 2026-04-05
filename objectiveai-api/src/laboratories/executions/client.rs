use crate::ctx;
use futures::Stream;

/// Trait for laboratory execution clients.
///
/// Implementations handle the full lifecycle: spinning up builder containers,
/// running agent completions via MCP, evaluating results, and reporting usage.
pub trait LaboratoryClient<CTXEXT>: Send + Sync + 'static {
    type Error: objectiveai::error::StatusError + Send + Sync + 'static;

    fn create_unary_handle_usage(
        self: std::sync::Arc<Self>,
        ctx: ctx::Context<CTXEXT, impl ctx::persistent_cache::PersistentCacheClient>,
        request: std::sync::Arc<
            objectiveai::laboratories::executions::request::LaboratoryExecutionCreateParams,
        >,
    ) -> impl std::future::Future<
        Output = Result<
            objectiveai::laboratories::executions::response::unary::LaboratoryExecution,
            Self::Error,
        >,
    > + Send;

    fn create_streaming_handle_usage(
        self: std::sync::Arc<Self>,
        ctx: ctx::Context<CTXEXT, impl ctx::persistent_cache::PersistentCacheClient>,
        request: std::sync::Arc<
            objectiveai::laboratories::executions::request::LaboratoryExecutionCreateParams,
        >,
    ) -> impl std::future::Future<
        Output = Result<
            impl Stream<
                    Item = objectiveai::laboratories::executions::response::streaming::LaboratoryExecutionChunk,
                > + Send
                + Unpin
                + 'static,
            Self::Error,
        >,
    > + Send;
}
