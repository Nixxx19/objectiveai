//! WebSocket variants of the 8 streaming endpoints (stage 1 of #193).
//!
//! Each `_ws` handler mirrors the corresponding SSE handler in
//! [`crate::run`] (same `create_streaming_handle_usage` call with the
//! same args) but consumes the resulting stream via
//! [`crate::streaming_ws::serve_chunks`] — one JSON text frame per
//! chunk, then `Close(1000)`. Errors during setup land as
//! `Close(1011)` after a `ResponseError` text frame; body-deserialize
//! failures land as `Close(1003)`.
//!
//! The `stream` field on the request body is ignored on this path —
//! opening a WS implies streaming intent.

use axum::extract::ws::{WebSocketUpgrade, close_code};
use objectiveai_sdk::error::ResponseError;
use std::sync::Arc;
use tokio_stream::StreamExt;

use crate::{
    agent, ctx, functions, retrieval, streaming_ws, vector,
};
use crate::functions::profiles::computations::Client as _;

pub(crate) async fn create_agent_completion_ws(
    client: Arc<
        agent::completions::Client<
            ctx::DefaultContextExt,
            impl agent::completions::UpstreamClient<
                objectiveai_sdk::agent::openrouter::Agent, objectiveai_sdk::agent::openrouter::Continuation,
            > + Send + Sync + 'static,
            impl agent::completions::UpstreamClient<
                objectiveai_sdk::agent::claude_agent_sdk::Agent, objectiveai_sdk::agent::claude_agent_sdk::Continuation,
            > + Send + Sync + 'static,
            impl agent::completions::UpstreamClient<
                objectiveai_sdk::agent::codex_sdk::Agent, objectiveai_sdk::agent::codex_sdk::Continuation,
            > + Send + Sync + 'static,
            impl agent::completions::UpstreamClient<
                objectiveai_sdk::agent::mock::Agent, objectiveai_sdk::agent::mock::Continuation,
            > + Send + Sync + 'static,
            impl retrieval::retrieve::Client<ctx::DefaultContextExt> + Send + Sync + 'static,
            impl retrieval::retrieve::Client<ctx::DefaultContextExt> + Send + Sync + 'static,
            impl retrieval::retrieve::Client<ctx::DefaultContextExt> + Send + Sync + 'static,
            impl agent::completions::usage_handler::UsageHandler<ctx::DefaultContextExt> + Send + Sync + 'static,
        >,
    >,
    headers: axum::http::HeaderMap,
    persistent_cache: Arc<impl ctx::persistent_cache::PersistentCacheClient + 'static>,
    suppress_output: bool,
    ws: WebSocketUpgrade,
) -> axum::response::Response {
    ws.on_upgrade(move |mut socket| async move {
        let body: objectiveai_sdk::agent::completions::request::AgentCompletionCreateParams =
            match streaming_ws::recv_body_frame(&mut socket).await {
                Ok(b) => b,
                Err(err) => {
                    streaming_ws::send_error_and_close(&mut socket, &err, close_code::UNSUPPORTED)
                        .await;
                    return;
                }
            };
        let ctx = crate::context(&headers, persistent_cache, suppress_output);
        match client
            .create_streaming_handle_usage(
                ctx,
                Arc::new(body),
                None,
                None, // disable_tools
                vec![], // extra_mcp_servers
                indexmap::IndexMap::new(), // extra_mcp_headers
                None,
                true,
                None,
                None,
                None,
                None,
            )
            .await
        {
            Ok(stream) => {
                let chunks = stream.filter_map(|item| match item {
                    agent::completions::StreamItem::Chunk(c) => Some(c),
                    agent::completions::StreamItem::State(_) => None,
                });
                streaming_ws::serve_chunks(&mut socket, Box::pin(chunks)).await;
            }
            Err(e) => {
                streaming_ws::fatal_setup_error(&mut socket, &ResponseError::from(&e)).await;
            }
        }
    })
}

pub(crate) async fn create_vector_completion_ws(
    client: Arc<
        vector::completions::Client<
            ctx::DefaultContextExt,
            impl agent::completions::UpstreamClient<
                objectiveai_sdk::agent::openrouter::Agent, objectiveai_sdk::agent::openrouter::Continuation,
            > + Send + Sync + 'static,
            impl agent::completions::UpstreamClient<
                objectiveai_sdk::agent::claude_agent_sdk::Agent, objectiveai_sdk::agent::claude_agent_sdk::Continuation,
            > + Send + Sync + 'static,
            impl agent::completions::UpstreamClient<
                objectiveai_sdk::agent::codex_sdk::Agent, objectiveai_sdk::agent::codex_sdk::Continuation,
            > + Send + Sync + 'static,
            impl agent::completions::UpstreamClient<
                objectiveai_sdk::agent::mock::Agent, objectiveai_sdk::agent::mock::Continuation,
            > + Send + Sync + 'static,
            impl retrieval::retrieve::Client<ctx::DefaultContextExt> + Send + Sync + 'static,
            impl retrieval::retrieve::Client<ctx::DefaultContextExt> + Send + Sync + 'static,
            impl retrieval::retrieve::Client<ctx::DefaultContextExt> + Send + Sync + 'static,
            impl agent::completions::usage_handler::UsageHandler<ctx::DefaultContextExt> + Send + Sync + 'static,
            impl vector::completions::completion_votes_fetcher::Fetcher<ctx::DefaultContextExt> + Send + Sync + 'static,
            impl vector::completions::cache_vote_fetcher::Fetcher<ctx::DefaultContextExt> + Send + Sync + 'static,
            impl vector::completions::usage_handler::UsageHandler<ctx::DefaultContextExt> + Send + Sync + 'static,
        >,
    >,
    headers: axum::http::HeaderMap,
    persistent_cache: Arc<impl ctx::persistent_cache::PersistentCacheClient + 'static>,
    suppress_output: bool,
    ws: WebSocketUpgrade,
) -> axum::response::Response {
    ws.on_upgrade(move |mut socket| async move {
        let body: objectiveai_sdk::vector::completions::request::VectorCompletionCreateParams =
            match streaming_ws::recv_body_frame(&mut socket).await {
                Ok(b) => b,
                Err(err) => {
                    streaming_ws::send_error_and_close(&mut socket, &err, close_code::UNSUPPORTED)
                        .await;
                    return;
                }
            };
        let ctx = crate::context(&headers, persistent_cache, suppress_output);
        match client.create_streaming_handle_usage(ctx, Arc::new(body)).await {
            Ok(stream) => {
                streaming_ws::serve_chunks(&mut socket, Box::pin(stream)).await;
            }
            Err(e) => {
                streaming_ws::fatal_setup_error(&mut socket, &ResponseError::from(&e)).await;
            }
        }
    })
}

pub(crate) async fn execute_function_ws(
    client: Arc<
        functions::executions::Client<
            ctx::DefaultContextExt,
            impl agent::completions::UpstreamClient<
                objectiveai_sdk::agent::openrouter::Agent, objectiveai_sdk::agent::openrouter::Continuation,
            > + Send + Sync + 'static,
            impl agent::completions::UpstreamClient<
                objectiveai_sdk::agent::claude_agent_sdk::Agent, objectiveai_sdk::agent::claude_agent_sdk::Continuation,
            > + Send + Sync + 'static,
            impl agent::completions::UpstreamClient<
                objectiveai_sdk::agent::codex_sdk::Agent, objectiveai_sdk::agent::codex_sdk::Continuation,
            > + Send + Sync + 'static,
            impl agent::completions::UpstreamClient<
                objectiveai_sdk::agent::mock::Agent, objectiveai_sdk::agent::mock::Continuation,
            > + Send + Sync + 'static,
            impl agent::completions::usage_handler::UsageHandler<ctx::DefaultContextExt> + Send + Sync + 'static,
            impl vector::completions::completion_votes_fetcher::Fetcher<ctx::DefaultContextExt> + Send + Sync + 'static,
            impl vector::completions::cache_vote_fetcher::Fetcher<ctx::DefaultContextExt> + Send + Sync + 'static,
            impl vector::completions::usage_handler::UsageHandler<ctx::DefaultContextExt> + Send + Sync + 'static,
            impl retrieval::retrieve::Client<ctx::DefaultContextExt> + Send + Sync + 'static,
            impl retrieval::retrieve::Client<ctx::DefaultContextExt> + Send + Sync + 'static,
            impl retrieval::retrieve::Client<ctx::DefaultContextExt> + Send + Sync + 'static,
            impl functions::executions::usage_handler::UsageHandler<ctx::DefaultContextExt> + Send + Sync + 'static,
        >,
    >,
    headers: axum::http::HeaderMap,
    persistent_cache: Arc<impl ctx::persistent_cache::PersistentCacheClient + 'static>,
    suppress_output: bool,
    ws: WebSocketUpgrade,
) -> axum::response::Response {
    ws.on_upgrade(move |mut socket| async move {
        let body: objectiveai_sdk::functions::executions::request::FunctionExecutionCreateParams =
            match streaming_ws::recv_body_frame(&mut socket).await {
                Ok(b) => b,
                Err(err) => {
                    streaming_ws::send_error_and_close(&mut socket, &err, close_code::UNSUPPORTED)
                        .await;
                    return;
                }
            };
        let ctx = crate::context(&headers, persistent_cache, suppress_output);
        match client.create_streaming_handle_usage(ctx, Arc::new(body)).await {
            Ok(stream) => {
                streaming_ws::serve_chunks(&mut socket, Box::pin(stream)).await;
            }
            Err(e) => {
                streaming_ws::fatal_setup_error(&mut socket, &ResponseError::from(&e)).await;
            }
        }
    })
}

pub(crate) async fn create_profile_computation_ws(
    client: Arc<functions::profiles::computations::ObjectiveAiClient>,
    headers: axum::http::HeaderMap,
    persistent_cache: Arc<impl ctx::persistent_cache::PersistentCacheClient + 'static>,
    suppress_output: bool,
    ws: WebSocketUpgrade,
) -> axum::response::Response {
    ws.on_upgrade(move |mut socket| async move {
        let body: objectiveai_sdk::functions::profiles::computations::request::FunctionProfileComputationCreateParams =
            match streaming_ws::recv_body_frame(&mut socket).await {
                Ok(b) => b,
                Err(err) => {
                    streaming_ws::send_error_and_close(&mut socket, &err, close_code::UNSUPPORTED)
                        .await;
                    return;
                }
            };
        let ctx = crate::context(&headers, persistent_cache, suppress_output);
        match client.create_streaming(ctx, Arc::new(body)).await {
            Ok(stream) => {
                streaming_ws::serve_result_chunks(&mut socket, Box::pin(stream)).await;
            }
            Err(e) => {
                streaming_ws::fatal_setup_error(&mut socket, &e).await;
            }
        }
    })
}

pub(crate) async fn create_function_invention_ws(
    client: Arc<
        functions::inventions::Client<
            ctx::DefaultContextExt,
            impl agent::completions::UpstreamClient<
                objectiveai_sdk::agent::openrouter::Agent, objectiveai_sdk::agent::openrouter::Continuation,
            > + Send + Sync + 'static,
            impl agent::completions::UpstreamClient<
                objectiveai_sdk::agent::claude_agent_sdk::Agent, objectiveai_sdk::agent::claude_agent_sdk::Continuation,
            > + Send + Sync + 'static,
            impl agent::completions::UpstreamClient<
                objectiveai_sdk::agent::codex_sdk::Agent, objectiveai_sdk::agent::codex_sdk::Continuation,
            > + Send + Sync + 'static,
            impl agent::completions::UpstreamClient<
                objectiveai_sdk::agent::mock::Agent, objectiveai_sdk::agent::mock::Continuation,
            > + Send + Sync + 'static,
            impl retrieval::retrieve::Client<ctx::DefaultContextExt> + Send + Sync + 'static,
            impl retrieval::retrieve::Client<ctx::DefaultContextExt> + Send + Sync + 'static,
            impl retrieval::retrieve::Client<ctx::DefaultContextExt> + Send + Sync + 'static,
            impl agent::completions::usage_handler::UsageHandler<ctx::DefaultContextExt> + Send + Sync + 'static,
            impl functions::inventions::usage_handler::UsageHandler<ctx::DefaultContextExt> + Send + Sync + 'static,
            impl retrieval::retrieve::Client<ctx::DefaultContextExt> + Send + Sync + 'static,
            impl retrieval::retrieve::Client<ctx::DefaultContextExt> + Send + Sync + 'static,
            impl retrieval::retrieve::Client<ctx::DefaultContextExt> + Send + Sync + 'static,
        >,
    >,
    headers: axum::http::HeaderMap,
    persistent_cache: Arc<impl ctx::persistent_cache::PersistentCacheClient + 'static>,
    suppress_output: bool,
    ws: WebSocketUpgrade,
) -> axum::response::Response {
    ws.on_upgrade(move |mut socket| async move {
        let body: objectiveai_sdk::functions::inventions::request::FunctionInventionCreateParams =
            match streaming_ws::recv_body_frame(&mut socket).await {
                Ok(b) => b,
                Err(err) => {
                    streaming_ws::send_error_and_close(&mut socket, &err, close_code::UNSUPPORTED)
                        .await;
                    return;
                }
            };
        let ctx = crate::context(&headers, persistent_cache, suppress_output);
        match client.create_streaming_handle_usage(ctx, Arc::new(body)).await {
            Ok(stream) => {
                streaming_ws::serve_chunks(&mut socket, Box::pin(stream)).await;
            }
            Err(e) => {
                streaming_ws::fatal_setup_error(&mut socket, &ResponseError::from(&e)).await;
            }
        }
    })
}

pub(crate) async fn create_function_invention_recursive_ws(
    client: Arc<
        functions::inventions::recursive::Client<
            ctx::DefaultContextExt,
            impl agent::completions::UpstreamClient<
                objectiveai_sdk::agent::openrouter::Agent, objectiveai_sdk::agent::openrouter::Continuation,
            > + Send + Sync + 'static,
            impl agent::completions::UpstreamClient<
                objectiveai_sdk::agent::claude_agent_sdk::Agent, objectiveai_sdk::agent::claude_agent_sdk::Continuation,
            > + Send + Sync + 'static,
            impl agent::completions::UpstreamClient<
                objectiveai_sdk::agent::codex_sdk::Agent, objectiveai_sdk::agent::codex_sdk::Continuation,
            > + Send + Sync + 'static,
            impl agent::completions::UpstreamClient<
                objectiveai_sdk::agent::mock::Agent, objectiveai_sdk::agent::mock::Continuation,
            > + Send + Sync + 'static,
            impl retrieval::retrieve::Client<ctx::DefaultContextExt> + Send + Sync + 'static,
            impl retrieval::retrieve::Client<ctx::DefaultContextExt> + Send + Sync + 'static,
            impl retrieval::retrieve::Client<ctx::DefaultContextExt> + Send + Sync + 'static,
            impl agent::completions::usage_handler::UsageHandler<ctx::DefaultContextExt> + Send + Sync + 'static,
            impl functions::inventions::usage_handler::UsageHandler<ctx::DefaultContextExt> + Send + Sync + 'static,
            impl retrieval::retrieve::Client<ctx::DefaultContextExt> + Send + Sync + 'static,
            impl retrieval::retrieve::Client<ctx::DefaultContextExt> + Send + Sync + 'static,
            impl retrieval::retrieve::Client<ctx::DefaultContextExt> + Send + Sync + 'static,
            impl functions::inventions::recursive::usage_handler::UsageHandler<ctx::DefaultContextExt> + Send + Sync + 'static,
        >,
    >,
    headers: axum::http::HeaderMap,
    persistent_cache: Arc<impl ctx::persistent_cache::PersistentCacheClient + 'static>,
    suppress_output: bool,
    ws: WebSocketUpgrade,
) -> axum::response::Response {
    ws.on_upgrade(move |mut socket| async move {
        let body: objectiveai_sdk::functions::inventions::recursive::request::FunctionInventionRecursiveCreateParams =
            match streaming_ws::recv_body_frame(&mut socket).await {
                Ok(b) => b,
                Err(err) => {
                    streaming_ws::send_error_and_close(&mut socket, &err, close_code::UNSUPPORTED)
                        .await;
                    return;
                }
            };
        let ctx = crate::context(&headers, persistent_cache, suppress_output);
        match client.create_streaming_handle_usage(ctx, Arc::new(body)).await {
            Ok(stream) => {
                streaming_ws::serve_chunks(&mut socket, Box::pin(stream)).await;
            }
            Err(e) => {
                streaming_ws::fatal_setup_error(&mut socket, &ResponseError::from(&e)).await;
            }
        }
    })
}

pub(crate) async fn create_error_ws(
    client: Arc<crate::error::Client>,
    headers: axum::http::HeaderMap,
    persistent_cache: Arc<impl ctx::persistent_cache::PersistentCacheClient + 'static>,
    suppress_output: bool,
    ws: WebSocketUpgrade,
) -> axum::response::Response {
    ws.on_upgrade(move |mut socket| async move {
        let body: objectiveai_sdk::error::request::ErrorCreateParams =
            match streaming_ws::recv_body_frame(&mut socket).await {
                Ok(b) => b,
                Err(err) => {
                    streaming_ws::send_error_and_close(&mut socket, &err, close_code::UNSUPPORTED)
                        .await;
                    return;
                }
            };
        let ctx = crate::context(&headers, persistent_cache, suppress_output);
        match client.create_streaming(&ctx, &body) {
            Ok(stream) => {
                streaming_ws::serve_result_chunks(&mut socket, Box::pin(stream)).await;
            }
            Err(e) => {
                streaming_ws::fatal_setup_error(&mut socket, &e).await;
            }
        }
    })
}

pub(crate) async fn execute_laboratory_ws(
    client: Arc<
        crate::laboratories::executions::Client<
            ctx::DefaultContextExt,
            impl agent::completions::UpstreamClient<
                objectiveai_sdk::agent::openrouter::Agent, objectiveai_sdk::agent::openrouter::Continuation,
            > + Send + Sync + 'static,
            impl agent::completions::UpstreamClient<
                objectiveai_sdk::agent::claude_agent_sdk::Agent, objectiveai_sdk::agent::claude_agent_sdk::Continuation,
            > + Send + Sync + 'static,
            impl agent::completions::UpstreamClient<
                objectiveai_sdk::agent::codex_sdk::Agent, objectiveai_sdk::agent::codex_sdk::Continuation,
            > + Send + Sync + 'static,
            impl agent::completions::UpstreamClient<
                objectiveai_sdk::agent::mock::Agent, objectiveai_sdk::agent::mock::Continuation,
            > + Send + Sync + 'static,
            impl retrieval::retrieve::Client<ctx::DefaultContextExt> + Send + Sync + 'static,
            impl retrieval::retrieve::Client<ctx::DefaultContextExt> + Send + Sync + 'static,
            impl retrieval::retrieve::Client<ctx::DefaultContextExt> + Send + Sync + 'static,
            impl agent::completions::usage_handler::UsageHandler<ctx::DefaultContextExt> + Send + Sync + 'static,
            impl crate::laboratories::executions::usage_handler::UsageHandler<ctx::DefaultContextExt> + Send + Sync + 'static,
            impl crate::laboratories::orchestrator::Orchestrator<ctx::DefaultContextExt> + Send + Sync + 'static,
        >,
    >,
    headers: axum::http::HeaderMap,
    persistent_cache: Arc<impl ctx::persistent_cache::PersistentCacheClient + 'static>,
    suppress_output: bool,
    ws: WebSocketUpgrade,
) -> axum::response::Response {
    ws.on_upgrade(move |mut socket| async move {
        let request: objectiveai_sdk::laboratories::executions::request::LaboratoryExecutionCreateParams =
            match streaming_ws::recv_body_frame(&mut socket).await {
                Ok(b) => b,
                Err(err) => {
                    streaming_ws::send_error_and_close(&mut socket, &err, close_code::UNSUPPORTED)
                        .await;
                    return;
                }
            };
        let ctx = crate::context(&headers, persistent_cache, suppress_output);
        match client
            .create_streaming_handle_usage(ctx, Arc::new(request))
            .await
        {
            Ok(stream) => {
                streaming_ws::serve_chunks(&mut socket, Box::pin(stream)).await;
            }
            Err(e) => {
                streaming_ws::fatal_setup_error(&mut socket, &ResponseError::from(&e)).await;
            }
        }
    })
}
