//! Mock upstream client for agent completions.

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;
use futures::Stream;
use rand::{Rng, SeedableRng};
use super::super::{ContinuationItem, StreamItem, UpstreamClient, ResolvedTool};

/// Mock upstream client that generates random responses with configurable delay.
#[derive(Debug)]
pub struct Client {
    /// Delay before yielding each chunk.
    pub delay: Duration,
    /// Optional RNG seed for deterministic output.
    pub seed: Option<u64>,
    /// Optional maximum number of tool calls across all continuations.
    /// When the counter reaches this limit, the mock will always respond
    /// with content instead of a tool call.
    pub max_tool_calls: Option<u32>,
    /// Shared counter tracking how many tool calls have been made.
    pub tool_call_count: Arc<AtomicU32>,
}

impl Clone for Client {
    fn clone(&self) -> Self {
        Self {
            delay: self.delay,
            seed: self.seed,
            max_tool_calls: self.max_tool_calls,
            tool_call_count: self.tool_call_count.clone(),
        }
    }
}

/// Resolves the response format for this agent from the request params.
fn resolve_response_format(
    agent_id: &str,
    params: &objectiveai::agent::completions::request::AgentCompletionCreateParams,
) -> Option<objectiveai::agent::completions::request::ResponseFormat> {
    use objectiveai::agent::completions::request::ResponseFormatParam;
    match params.response_format.as_ref()? {
        ResponseFormatParam::Single(rf) => Some(rf.clone()),
        ResponseFormatParam::PerAgent(map) => map.get(agent_id).cloned(),
    }
}

/// The outcome of the tool-vs-content dice roll.
enum MockResponse {
    /// Respond with text content, split across N chunks.
    Content {
        text: String,
        n_chunks: usize,
    },
    /// Respond with a tool call, arguments split across N delta chunks.
    ToolCall {
        tool_name: String,
        call_id: String,
        arguments: String,
        n_deltas: usize,
    },
}

impl UpstreamClient<objectiveai::agent::mock::Agent> for Client {
    type State = ();
    type Stream = Pin<
        Box<dyn Stream<Item = StreamItem<Self::State>> + Send + 'static>,
    >;

    fn create(
        &self,
        id: &str,
        created: u64,
        agent: &objectiveai::agent::mock::Agent,
        params: &objectiveai::agent::completions::request::AgentCompletionCreateParams,
        _messages: &[objectiveai::agent::completions::message::Message],
        _mcp_connections: &[Arc<crate::mcp::Connection>],
        _invention_tools: Option<
            &[objectiveai::functions::inventions::InventionTool],
        >,
        tool_names: &[String],
        tool_map: &HashMap<String, ResolvedTool>,
        _continuation: Option<&[ContinuationItem<Self::State>]>,
        byok: Option<&str>,
        _cost_multiplier: rust_decimal::Decimal,
    ) -> impl Future<
        Output = Result<
            (Self::Stream, Self::State),
            objectiveai::error::ResponseError,
        >,
    > + Send
    + 'static {
        let id = id.to_string();
        let agent_id = agent.id.clone();
        let error = agent.base.error == Some(true);
        let response_format = resolve_response_format(&agent.id, params);
        let tool_names = tool_names.to_vec();
        let tool_map = tool_map.clone();
        let delay = self.delay;
        let cont_len = _continuation.map_or(0u64, |c| c.len() as u64);
        let seed = self.seed.map(|s| s.wrapping_add(cont_len));
        let max_tool_calls = self.max_tool_calls;
        let tool_call_count = self.tool_call_count.clone();
        let is_byok = byok.is_some();

        async move {
            use objectiveai::agent::completions::request::ResponseFormat;

            if error {
                return Err(objectiveai::error::ResponseError {
                    code: 500,
                    message: serde_json::json!("Expected error"),
                });
            }

            // Reject Grammar and Python response formats.
            if let Some(ref rf) = response_format {
                match rf {
                    ResponseFormat::Grammar { .. } | ResponseFormat::Python => {
                        return Err(objectiveai::error::ResponseError {
                            code: 400,
                            message: serde_json::json!({
                                "kind": "invalid_response_format",
                                "error": "mock client does not support grammar or python response formats",
                            }),
                        });
                    }
                    _ => {}
                }
            }

            let mut rng = match seed {
                Some(s) => rand::rngs::StdRng::seed_from_u64(s),
                None => rand::rngs::StdRng::from_os_rng(),
            };

            // --- Reasoning: roll 0-5 chunks ---
            let n_reasoning = rng.random_range(0u32..=5);
            let reasoning_chunks: Vec<String> = (0..n_reasoning)
                .map(|_| random_string(&mut rng, 20, 200))
                .collect();

            // --- Tool call vs content ---
            let tools_exhausted = match max_tool_calls {
                Some(max) => tool_call_count.load(Ordering::Relaxed) >= max,
                None => false,
            };
            let mock_response = resolve_mock_response(
                &response_format,
                if tools_exhausted { &[] } else { &tool_names },
                &tool_map,
                &mut rng,
            );
            if matches!(&mock_response, MockResponse::ToolCall { .. }) {
                tool_call_count.fetch_add(1, Ordering::Relaxed);
            }

            let stream = async_stream::stream! {
                use objectiveai::agent::completions::message::{
                    AssistantToolCallDelta, AssistantToolCallFunctionDelta,
                    AssistantToolCallType, RichContent,
                };
                use objectiveai::agent::completions::response::streaming::{
                    AgentCompletionChunk, AssistantResponseChunk, MessageChunk,
                };
                use objectiveai::agent::completions::response::FinishReason;

                // --- Yield reasoning chunks ---
                for reasoning_text in &reasoning_chunks {
                    if !delay.is_zero() {
                        tokio::time::sleep(delay).await;
                    }
                    yield StreamItem::Chunk(AgentCompletionChunk {
                        id: id.clone(),
                        created,
                        messages: vec![MessageChunk::Assistant(AssistantResponseChunk {
                            index: 0,
                            created,
                            agent: agent_id.clone(),
                            model: "mock".into(),
                            upstream_id: id.clone(),
                            reasoning: Some(reasoning_text.clone()),
                            ..Default::default()
                        })],
                        ..Default::default()
                    });
                }

                // --- Yield content or tool call chunks ---
                match &mock_response {
                    MockResponse::Content { text, n_chunks } => {
                        let chunk_size = (text.len() + n_chunks - 1) / n_chunks;
                        let parts: Vec<&str> = if text.is_empty() {
                            vec![""]
                        } else {
                            text.as_bytes()
                                .chunks(chunk_size.max(1))
                                .map(|b| std::str::from_utf8(b).unwrap_or(""))
                                .collect()
                        };

                        for (i, part) in parts.iter().enumerate() {
                            let is_last = i == parts.len() - 1;
                            if !delay.is_zero() {
                                tokio::time::sleep(delay).await;
                            }
                            yield StreamItem::Chunk(AgentCompletionChunk {
                                id: id.clone(),
                                created,
                                messages: vec![MessageChunk::Assistant(AssistantResponseChunk {
                                    index: 0,
                                    created,
                                    agent: agent_id.clone(),
                                    model: "mock".into(),
                                    upstream_id: id.clone(),
                                    content: Some(RichContent::Text(part.to_string())),
                                    finish_reason: if is_last {
                                        Some(FinishReason::Stop)
                                    } else {
                                        None
                                    },
                                    ..Default::default()
                                })],
                                ..Default::default()
                            });
                        }
                    }
                    MockResponse::ToolCall { tool_name, call_id, arguments, n_deltas } => {
                        let chunk_size = (arguments.len() + n_deltas - 1) / n_deltas;
                        let parts: Vec<&str> = if arguments.is_empty() {
                            vec![""]
                        } else {
                            arguments.as_bytes()
                                .chunks(chunk_size.max(1))
                                .map(|b| std::str::from_utf8(b).unwrap_or(""))
                                .collect()
                        };

                        for (i, part) in parts.iter().enumerate() {
                            let is_first = i == 0;
                            let is_last = i == parts.len() - 1;
                            if !delay.is_zero() {
                                tokio::time::sleep(delay).await;
                            }
                            yield StreamItem::Chunk(AgentCompletionChunk {
                                id: id.clone(),
                                created,
                                messages: vec![MessageChunk::Assistant(AssistantResponseChunk {
                                    index: 0,
                                    created,
                                    agent: agent_id.clone(),
                                    model: "mock".into(),
                                    upstream_id: id.clone(),
                                    tool_calls: Some(vec![AssistantToolCallDelta {
                                        index: 0,
                                        r#type: if is_first {
                                            Some(AssistantToolCallType::Function)
                                        } else {
                                            None
                                        },
                                        id: if is_first {
                                            Some(call_id.clone())
                                        } else {
                                            None
                                        },
                                        function: Some(AssistantToolCallFunctionDelta {
                                            name: if is_first {
                                                Some(tool_name.clone())
                                            } else {
                                                None
                                            },
                                            arguments: Some(part.to_string()),
                                        }),
                                    }]),
                                    finish_reason: if is_last {
                                        Some(FinishReason::ToolCalls)
                                    } else {
                                        None
                                    },
                                    ..Default::default()
                                })],
                                ..Default::default()
                            });
                        }
                    }
                }

                // --- Yield usage chunk ---
                if !delay.is_zero() {
                    tokio::time::sleep(delay).await;
                }
                yield StreamItem::Chunk(AgentCompletionChunk {
                    id: id.clone(),
                    created,
                    usage: Some(objectiveai::agent::completions::response::Usage {
                        is_byok,
                        ..Default::default()
                    }),
                    ..Default::default()
                });

                // --- Yield final state ---
                yield StreamItem::State(());
            };

            let boxed: Pin<Box<dyn Stream<Item = StreamItem<Self::State>> + Send>> =
                Box::pin(stream);
            Ok((boxed, ()))
        }
    }
}

/// Decides whether to call a tool or respond with content, and generates the data.
fn resolve_mock_response(
    response_format: &Option<objectiveai::agent::completions::request::ResponseFormat>,
    tool_names: &[String],
    tool_map: &HashMap<String, ResolvedTool>,
    rng: &mut impl Rng,
) -> MockResponse {
    use objectiveai::agent::completions::request::ResponseFormat;

    // Check for required tool call from response format.
    if let Some(ResponseFormat::ToolCall {
        name, required: Some(true), ..
    }) = response_format
    {
        if tool_map.contains_key(name) {
            let arguments = generate_tool_arguments(tool_map, name, rng);
            return MockResponse::ToolCall {
                tool_name: name.clone(),
                call_id: format!("call_mock_{}", rng.random_range(0u64..u64::MAX)),
                arguments,
                n_deltas: rng.random_range(1u32..=5) as usize,
            };
        }
    }

    if !tool_names.is_empty() {
        // Roll the dice: equal probability for each tool or respond as-is,
        // with respond-as-is having a minimum 25% chance.
        let n_tools = tool_names.len();
        let respond_as_is_weight = if n_tools >= 3 {
            25u32
        } else {
            (100 / (n_tools as u32 + 1)).max(25)
        };
        let tool_weight = (100 - respond_as_is_weight) / n_tools as u32;
        let roll = rng.random_range(0u32..100);

        if roll >= respond_as_is_weight {
            let tool_index = ((roll - respond_as_is_weight) / tool_weight.max(1))
                .min(n_tools as u32 - 1) as usize;
            let tool_name = &tool_names[tool_index];
            let arguments = generate_tool_arguments(tool_map, tool_name, rng);
            return MockResponse::ToolCall {
                tool_name: tool_name.clone(),
                call_id: format!("call_mock_{}", rng.random_range(0u64..u64::MAX)),
                arguments,
                n_deltas: rng.random_range(1u32..=5) as usize,
            };
        }
    }

    // Respond as-is with content.
    let text = generate_content_string(response_format, rng);
    MockResponse::Content {
        text,
        n_chunks: rng.random_range(1u32..=5) as usize,
    }
}

/// Generates the content string for a respond-as-is case based on response format.
fn generate_content_string(
    response_format: &Option<objectiveai::agent::completions::request::ResponseFormat>,
    rng: &mut impl Rng,
) -> String {
    use objectiveai::agent::completions::request::ResponseFormat;

    match response_format {
        Some(ResponseFormat::JsonObject) => "{}".into(),
        Some(ResponseFormat::JsonSchema { schema }) | Some(ResponseFormat::ToolCall { schema, .. }) => {
            generate_from_schema(schema, rng)
        }
        _ => random_string(rng, 10, 100),
    }
}

/// Generates a JSON string from an IndexMap schema, falling back to "{}".
fn generate_from_schema(
    schema: &indexmap::IndexMap<String, serde_json::Value>,
    rng: &mut impl Rng,
) -> String {
    match serde_json::from_value::<super::json_schema::JsonSchema>(
        serde_json::Value::Object(
            schema.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        ),
    ) {
        Ok(js) => {
            let value = js.generate_from_rng(rng);
            serde_json::to_string(&value).unwrap_or_else(|_| "{}".into())
        }
        Err(_) => "{}".into(),
    }
}

/// Generates a random alphanumeric string (with spaces) of length between `min` and `max`.
fn random_string(rng: &mut impl Rng, min: usize, max: usize) -> String {
    let len = rng.random_range(min..=max);
    (0..len)
        .map(|_| {
            const CHARS: &[u8] =
                b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 ";
            CHARS[rng.random_range(0..CHARS.len())] as char
        })
        .collect()
}

/// Generates tool call arguments by parsing the tool's parameter schema as JsonSchema.
fn generate_tool_arguments(
    tool_map: &HashMap<String, ResolvedTool>,
    tool_name: &str,
    rng: &mut impl Rng,
) -> String {
    let schema_value = match tool_map.get(tool_name) {
        Some(ResolvedTool::Mcp { tool, .. }) => {
            let mut map = serde_json::Map::new();
            map.insert("type".into(), serde_json::json!("object"));
            if let Some(props) = &tool.input_schema.properties {
                map.insert(
                    "properties".into(),
                    serde_json::Value::Object(
                        props.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
                    ),
                );
            }
            Some(serde_json::Value::Object(map))
        }
        Some(ResolvedTool::InventionTool(inv)) => {
            Some(serde_json::Value::Object(
                inv.parameters.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            ))
        }
        Some(ResolvedTool::ResponseFormat { schema, .. }) => {
            Some(serde_json::Value::Object(
                schema.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
            ))
        }
        None => None,
    };

    match schema_value {
        Some(sv) => {
            match serde_json::from_value::<super::json_schema::JsonSchema>(sv) {
                Ok(js) => {
                    let value = js.generate_from_rng(rng);
                    serde_json::to_string(&value).unwrap_or_else(|_| "{}".into())
                }
                Err(_) => "{}".into(),
            }
        }
        None => "{}".into(),
    }
}

#[cfg(test)]
#[path = "client_tests.rs"]
mod tests;
