//! HTTP functions for agent completions.

use crate::{HttpClient, HttpError};
use futures::Stream;

/// Creates a agent completion (non-streaming).
///
/// Sends a request to the agent completions endpoint and waits for the
/// complete response.
///
/// # Arguments
///
/// * `client` - The HTTP client to use
/// * `params` - Agent completion parameters (stream flag will be set to false)
///
/// # Returns
///
/// The complete agent completion response.
pub async fn create_agent_completion_unary(
    client: &HttpClient,
    mut params: super::request::AgentCompletionCreateParams,
) -> Result<super::response::unary::AgentCompletion, HttpError> {
    params.stream = None;
    client
        .send_unary(reqwest::Method::POST, "agent/completions", Some(params))
        .await
}

/// Creates a streaming agent completion.
///
/// Sends a request to the agent completions endpoint and returns a stream
/// of response chunks as they arrive via Server-Sent Events.
///
/// # Arguments
///
/// * `client` - The HTTP client to use
/// * `params` - Agent completion parameters (stream flag will be set to true)
///
/// # Returns
///
/// A stream of agent completion chunks.
pub async fn create_agent_completion_streaming(
    client: &HttpClient,
    mut params: super::request::AgentCompletionCreateParams,
) -> Result<
    impl Stream<
        Item = Result<
            super::response::streaming::AgentCompletionChunk,
            HttpError,
        >,
    >
    + Send
    + 'static
    + use<>,
    HttpError,
> {
    params.stream = Some(true);
    client
        .send_streaming(reqwest::Method::POST, "agent/completions", Some(params))
        .await
}
