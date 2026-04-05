use std::sync::Arc;

use futures::Stream;

use crate::ctx;

/// Error returned by the unimplemented laboratory client.
#[derive(Debug, thiserror::Error)]
#[error("laboratory executions are not available (enable the laboratories feature)")]
pub struct UnimplementedError;

impl objectiveai::error::StatusError for UnimplementedError {
    fn status(&self) -> u16 {
        501
    }

    fn message(&self) -> Option<serde_json::Value> {
        Some(serde_json::json!({
            "kind": "laboratory",
            "error": {
                "kind": "unimplemented",
                "error": "laboratory executions are not available (enable the laboratories feature)",
            }
        }))
    }
}

/// Stub client that always returns an unimplemented error.
pub struct UnimplementedClient;

impl<CTXEXT: Send + Sync + 'static> super::LaboratoryClient<CTXEXT> for UnimplementedClient {
    type Error = UnimplementedError;

    fn create_unary_handle_usage(
        self: Arc<Self>,
        _ctx: ctx::Context<CTXEXT, impl ctx::persistent_cache::PersistentCacheClient>,
        _request: Arc<
            objectiveai::laboratories::executions::request::LaboratoryExecutionCreateParams,
        >,
    ) -> impl std::future::Future<
        Output = Result<
            objectiveai::laboratories::executions::response::unary::LaboratoryExecution,
            UnimplementedError,
        >,
    > + Send {
        async { Err(UnimplementedError) }
    }

    fn create_streaming_handle_usage(
        self: Arc<Self>,
        _ctx: ctx::Context<CTXEXT, impl ctx::persistent_cache::PersistentCacheClient>,
        _request: Arc<
            objectiveai::laboratories::executions::request::LaboratoryExecutionCreateParams,
        >,
    ) -> impl std::future::Future<
        Output = Result<
            impl Stream<
                    Item = objectiveai::laboratories::executions::response::streaming::LaboratoryExecutionChunk,
                > + Send
                + Unpin
                + 'static,
            UnimplementedError,
        >,
    > + Send {
        async {
            Err::<futures::stream::Empty<_>, _>(UnimplementedError)
        }
    }
}
