//! OpenRouter HTTP client for agent completions.

use crate::agent::completions::{
    ContinuationItem, StreamItem, UpstreamClient,
};
use crate::util::StreamOnce;
use eventsource_stream::Event as MessageEvent;
use futures::{Stream, StreamExt};
use reqwest_eventsource::{Event, EventSource, RequestBuilderExt};
use std::pin::Pin;
use std::sync::Arc;

/// HTTP client for communicating with the OpenRouter API for agent completions.
#[derive(Debug, Clone)]
pub struct Client {
    /// The underlying HTTP client.
    pub http_client: reqwest::Client,
    /// Base URL for the OpenRouter API.
    pub api_base: String,
    /// API key for authentication with OpenRouter.
    pub api_key: String,
    /// Optional User-Agent header value.
    pub user_agent: Option<String>,
    /// Optional X-Title header value.
    pub x_title: Option<String>,
    /// Optional Referer header value (sent as both referer and http-referer).
    pub referer: Option<String>,
}

impl Client {
    /// Creates an SSE EventSource for the streaming request.
    fn create_streaming_event_source(
        &self,
        api_key: &str,
        request: &super::request::ChatCompletionCreateParams,
    ) -> EventSource {
        let mut http_request = self
            .http_client
            .post(format!("{}/chat/completions", self.api_base))
            .header("authorization", format!("Bearer {}", api_key));
        if let Some(ref user_agent) = self.user_agent {
            http_request = http_request.header("user-agent", user_agent);
        }
        if let Some(ref x_title) = self.x_title {
            http_request = http_request.header("x-title", x_title);
        }
        if let Some(ref referer) = self.referer {
            http_request = http_request
                .header("referer", referer)
                .header("http-referer", referer);
        }
        http_request.json(request).eventsource().unwrap()
    }

    /// Processes the SSE EventSource into a stream of agent completion chunks,
    /// followed by a final accumulated state.
    fn create_streaming_stream(
        mut event_source: EventSource,
        id: String,
        created: u64,
        agent: String,
        index: u64,
        is_byok: bool,
        cost_multiplier: rust_decimal::Decimal,
    ) -> impl Stream<
        Item = StreamItem<
            objectiveai::agent::completions::message::AssistantMessage,
        >,
    > + Send
    + 'static {
        async_stream::stream! {
            use objectiveai::agent::completions::message::AssistantMessage;
            use objectiveai::agent::completions::response::streaming::{
                AgentCompletionChunk, AssistantResponseChunk, MessageChunk,
            };

            let mut accumulated: Option<AssistantResponseChunk> = None;
            let mut had_error = false;

            while let Some(event) = event_source.next().await {
                match event {
                    Ok(Event::Open) => continue,
                    Ok(Event::Message(MessageEvent { data, .. })) => {
                        if data == "[DONE]" {
                            break;
                        } else if data.starts_with(":") {
                            continue;
                        } else if data.is_empty() {
                            continue;
                        }
                        let mut de =
                            serde_json::Deserializer::from_str(&data);
                        match serde_path_to_error::deserialize::<
                            _,
                            super::response::ChatCompletionChunk,
                        >(&mut de)
                        {
                            Ok(chunk) => {
                                let downstream = chunk.into_downstream(
                                    id.clone(),
                                    created,
                                    agent.clone(),
                                    index,
                                    is_byok,
                                    cost_multiplier,
                                );

                                // Accumulate the assistant response chunk.
                                for message in &downstream.messages {
                                    if let MessageChunk::Assistant(
                                        assistant_chunk,
                                    ) = message
                                    {
                                        match &mut accumulated {
                                            Some(acc) => {
                                                acc.push(assistant_chunk)
                                            }
                                            None => {
                                                accumulated = Some(
                                                    assistant_chunk.clone(),
                                                )
                                            }
                                        }
                                    }
                                }

                                yield StreamItem::Chunk(downstream);
                            }
                            Err(e) => {
                                // Try to parse as a provider error JSON,
                                // otherwise report as deserialization error.
                                let error = match serde_json::from_str::<serde_json::Value>(&data) {
                                    Ok(value) => {
                                        let code = value
                                            .pointer("/error/code")
                                            .and_then(|c| c.as_u64())
                                            .unwrap_or(500)
                                            as u16;
                                        objectiveai::error::ResponseError {
                                            code,
                                            message: serde_json::json!({
                                                "kind": "provider_error",
                                                "error": value,
                                            }),
                                        }
                                    }
                                    Err(_) => {
                                        objectiveai::error::ResponseError {
                                            code: 500,
                                            message: serde_json::json!({
                                                "kind": "deserialization",
                                                "error": e.to_string(),
                                            }),
                                        }
                                    }
                                };
                                yield StreamItem::Chunk(AgentCompletionChunk {
                                    id: id.clone(),
                                    error: Some(error),
                                    ..Default::default()
                                });
                                had_error = true;
                                break;
                            }
                        }
                    }
                    Err(reqwest_eventsource::Error::InvalidStatusCode(
                        code,
                        response,
                    )) => {
                        let body = match response.text().await {
                            Ok(body) => {
                                match serde_json::from_str::<
                                    serde_json::Value,
                                >(
                                    &body,
                                ) {
                                    Ok(value) => value,
                                    Err(_) => {
                                        serde_json::Value::String(body)
                                    }
                                }
                            }
                            Err(_) => serde_json::Value::Null,
                        };
                        yield StreamItem::Chunk(AgentCompletionChunk {
                            id: id.clone(),
                            error: Some(
                                objectiveai::error::ResponseError {
                                    code: code.as_u16(),
                                    message: serde_json::json!({
                                        "kind": "bad_status",
                                        "error": body,
                                    }),
                                },
                            ),
                            ..Default::default()
                        });
                        had_error = true;
                        break;
                    }
                    Err(e) => {
                        yield StreamItem::Chunk(AgentCompletionChunk {
                            id: id.clone(),
                            error: Some(
                                objectiveai::error::ResponseError {
                                    code: 500,
                                    message: serde_json::json!({
                                        "kind": "stream_error",
                                        "error": e.to_string(),
                                    }),
                                },
                            ),
                            ..Default::default()
                        });
                        had_error = true;
                        break;
                    }
                }
            }

            if !had_error {
                // Yield the final accumulated state.
                let state = match accumulated {
                    Some(acc) => AssistantMessage {
                        content: acc.content,
                        name: None,
                        refusal: acc.refusal,
                        tool_calls: acc.tool_calls.map(|tcs| {
                            tcs.into_iter().map(Into::into).collect()
                        }),
                        reasoning: acc.reasoning,
                    },
                    None => AssistantMessage {
                        content: None,
                        name: None,
                        refusal: None,
                        tool_calls: None,
                        reasoning: None,
                    },
                };
                yield StreamItem::State(state);
            }
        }
    }
}

impl UpstreamClient<objectiveai::agent::openrouter::Agent> for Client {
    type State = objectiveai::agent::completions::message::AssistantMessage;
    type Stream = Pin<
        Box<dyn Stream<Item = StreamItem<Self::State>> + Send + 'static>,
    >;

    fn create(
        &self,
        id: &str,
        created: u64,
        agent: &objectiveai::agent::openrouter::Agent,
        params: &objectiveai::agent::completions::request::AgentCompletionCreateParams,
        messages: &[objectiveai::agent::completions::message::Message],
        mcp_connections: &[Arc<crate::mcp::Connection>],
        invention_tools: Option<
            &[objectiveai::functions::inventions::InventionTool],
        >,
        tool_names: &[String],
        tool_map: &std::collections::HashMap<String, super::super::tool::ResolvedTool>,
        continuation: Option<&[ContinuationItem<Self::State>]>,
        byok: Option<&str>,
        cost_multiplier: rust_decimal::Decimal,
    ) -> impl Future<
        Output = Result<
            (Self::Stream, Self::State),
            objectiveai::error::ResponseError,
        >,
    > + Send
    + 'static {
        let id = id.to_string();
        let agent = agent.clone();
        let params = params.clone();
        let messages = messages.to_vec();
        let tool_names = tool_names.to_vec();
        let tool_map = tool_map.clone();
        let continuation = continuation.map(|c| c.to_vec());
        let client = self.clone();
        let is_byok = byok.is_some();
        let byok = byok.map(String::from);

        async move {
            let request =
                super::request::ChatCompletionCreateParams::new(
                    &agent,
                    &params,
                    &messages,
                    continuation.as_deref(),
                    &tool_names,
                    &tool_map,
                );

            let api_key = byok.as_deref().unwrap_or(&client.api_key);
            let event_source =
                client.create_streaming_event_source(api_key, &request);

            let index = continuation
                .as_deref()
                .map(|c| {
                    c.iter()
                        .filter(|item| {
                            matches!(
                                item,
                                ContinuationItem::State(_)
                                    | ContinuationItem::ToolMessage(_)
                            )
                        })
                        .count() as u64
                })
                .unwrap_or(0);

            let stream = Self::create_streaming_stream(
                event_source,
                id,
                created,
                agent.id.clone(),
                index,
                is_byok,
                cost_multiplier,
            );

            let initial_state = Self::State {
                content: None,
                name: None,
                refusal: None,
                tool_calls: None,
                reasoning: None,
            };

            // Await the first stream item. If it is an error chunk,
            // return Err so the caller never sees an error as the
            // first yielded item.
            let mut stream = Box::pin(stream);
            match stream.next().await {
                Some(StreamItem::Chunk(chunk)) if chunk.error.is_some() => {
                    return Err(chunk.error.unwrap());
                }
                Some(first) => {
                    let boxed: Pin<Box<dyn Stream<Item = StreamItem<Self::State>> + Send>> =
                        Box::pin(StreamOnce::new(first).chain(stream));
                    Ok((boxed, initial_state))
                }
                Some(StreamItem::State(_)) | None => {
                    return Err(objectiveai::error::ResponseError::from(
                        &super::Error::EmptyStream,
                    ));
                }
            }
        }
    }
}
