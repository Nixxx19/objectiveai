//! Trait for Profile computation (training).

use crate::ctx;
use futures::Stream;
use std::sync::Arc;

/// Client for computing (training) Profiles.
#[async_trait::async_trait]
pub trait Client<CTXEXT> {
    /// Computes a Profile and returns the complete result.
    async fn create_unary<PC: crate::ctx::persistent_cache::PersistentCacheClient>(
        &self,
        ctx: ctx::Context<CTXEXT, PC>,
        request: Arc<objectiveai::functions::profiles::computations::request::FunctionProfileComputationCreateParams>,
    ) -> Result<
        objectiveai::functions::profiles::computations::response::unary::FunctionProfileComputation,
        objectiveai::error::ResponseError,
    >;

    /// Computes a Profile with streaming progress updates.
    async fn create_streaming<PC: crate::ctx::persistent_cache::PersistentCacheClient>(
        &self,
        ctx: ctx::Context<CTXEXT, PC>,
        request: Arc<objectiveai::functions::profiles::computations::request::FunctionProfileComputationCreateParams>,
    ) -> Result<
        impl Stream<Item = Result<
            objectiveai::functions::profiles::computations::response::streaming::FunctionProfileComputationChunk,
            objectiveai::error::ResponseError,
        >>
            + Send
            + 'static,
        objectiveai::error::ResponseError,
    >;
}
