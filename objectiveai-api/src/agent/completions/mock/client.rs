//! Mock upstream client for agent completions.

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;

use futures::Stream;
use rand::Rng;

use super::super::tool::ResolvedTool;
use super::super::upstream_client::{ContinuationItem, StreamItem, UpstreamClient};

/// Mock upstream client that generates random responses.
#[derive(Debug, Clone)]
pub struct Client;

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
        _byok: Option<&str>,
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
        let response_format = resolve_response_format(&agent.id, params);
        let tool_names = tool_names.to_vec();
        let tool_map = tool_map.clone();

        async move {
            use objectiveai::agent::completions::message::{
                AssistantToolCallDelta, AssistantToolCallFunctionDelta,
                AssistantToolCallType, RichContent,
            };
            use objectiveai::agent::completions::request::ResponseFormat;
            use objectiveai::agent::completions::response::streaming::{
                AgentCompletionChunk, AssistantResponseChunk, MessageChunk,
            };
            use objectiveai::agent::completions::response::FinishReason;

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

            // Determine if we should call a tool or respond as-is.
            let mut rng = rand::rng();

            // Check for required tool call from response format.
            let required_tool = match &response_format {
                Some(ResponseFormat::ToolCall {
                    name, required: Some(true), ..
                }) => {
                    // Find this tool in the map.
                    if tool_map.contains_key(name) {
                        Some(name.clone())
                    } else {
                        None
                    }
                }
                _ => None,
            };

            let (finish_reason, content, tool_calls) = if let Some(ref tool_name) = required_tool {
                // Required tool call — always call it.
                let arguments = generate_tool_arguments(&tool_map, tool_name, &mut rng);
                (
                    FinishReason::ToolCalls,
                    None,
                    Some(vec![AssistantToolCallDelta {
                        index: 0,
                        r#type: Some(AssistantToolCallType::Function),
                        id: Some(format!("call_mock_{}", rng.random_range(0u64..u64::MAX))),
                        function: Some(AssistantToolCallFunctionDelta {
                            name: Some(tool_name.clone()),
                            arguments: Some(arguments),
                        }),
                    }]),
                )
            } else if !tool_names.is_empty() {
                // Roll the dice: equal probability for each tool or respond as-is,
                // with respond as-is having a minimum 25% chance.
                let n_tools = tool_names.len();
                let respond_as_is_weight = if n_tools >= 3 {
                    // 25% minimum for respond-as-is
                    25u32
                } else {
                    // Equal share: 1/(n_tools+1) as percentage
                    (100 / (n_tools as u32 + 1)).max(25)
                };
                let tool_weight = (100 - respond_as_is_weight) / n_tools as u32;
                let roll = rng.random_range(0u32..100);

                if roll < respond_as_is_weight {
                    // Respond as-is.
                    let content = generate_content(&response_format, &mut rng);
                    (FinishReason::Stop, content, None)
                } else {
                    // Pick a tool based on the roll.
                    let tool_index = ((roll - respond_as_is_weight) / tool_weight.max(1))
                        .min(n_tools as u32 - 1) as usize;
                    let tool_name = &tool_names[tool_index];
                    let arguments = generate_tool_arguments(&tool_map, tool_name, &mut rng);
                    (
                        FinishReason::ToolCalls,
                        None,
                        Some(vec![AssistantToolCallDelta {
                            index: 0,
                            r#type: Some(AssistantToolCallType::Function),
                            id: Some(format!("call_mock_{}", rng.random_range(0u64..u64::MAX))),
                            function: Some(AssistantToolCallFunctionDelta {
                                name: Some(tool_name.clone()),
                                arguments: Some(arguments),
                            }),
                        }]),
                    )
                }
            } else {
                // No tools — respond as-is.
                let content = generate_content(&response_format, &mut rng);
                (FinishReason::Stop, content, None)
            };

            // 50/50 chance to include reasoning.
            let reasoning = if rng.random_bool(0.5) {
                Some(random_string(&mut rng, 20, 200))
            } else {
                None
            };

            let chunk = AgentCompletionChunk {
                id: id.clone(),
                created,
                messages: vec![MessageChunk::Assistant(AssistantResponseChunk {
                    role: Default::default(),
                    index: 0,
                    created,
                    agent: agent_id,
                    model: "mock".into(),
                    upstream_id: id.clone(),
                    reasoning,
                    tool_calls,
                    content,
                    refusal: None,
                    finish_reason: Some(finish_reason),
                    logprobs: None,
                    service_tier: None,
                    system_fingerprint: None,
                    provider: None,
                })],
                object: Default::default(),
                usage: None,
                upstream: Default::default(),
                error: None,
            };

            let stream = async_stream::stream! {
                yield StreamItem::Chunk(chunk);
                yield StreamItem::State(());
            };

            let boxed: Pin<Box<dyn Stream<Item = StreamItem<Self::State>> + Send>> =
                Box::pin(stream);
            Ok((boxed, ()))
        }
    }
}

/// Generates content for a respond-as-is case based on response format.
fn generate_content(
    response_format: &Option<objectiveai::agent::completions::request::ResponseFormat>,
    rng: &mut impl Rng,
) -> Option<objectiveai::agent::completions::message::RichContent> {
    use objectiveai::agent::completions::message::RichContent;
    use objectiveai::agent::completions::request::ResponseFormat;

    match response_format {
        Some(ResponseFormat::JsonObject) => {
            Some(RichContent::Text("{}".into()))
        }
        Some(ResponseFormat::JsonSchema { schema }) => {
            match serde_json::from_value::<super::json_schema::JsonSchema>(
                serde_json::Value::Object(
                    schema.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
                ),
            ) {
                Ok(js) => {
                    let value = js.generate_from_rng(rng);
                    Some(RichContent::Text(serde_json::to_string(&value).unwrap_or_else(|_| "{}".into())))
                }
                Err(_) => Some(RichContent::Text("{}".into())),
            }
        }
        Some(ResponseFormat::ToolCall { schema, .. }) => {
            // If we're responding as-is with a non-required ToolCall format,
            // generate from the schema as content.
            match serde_json::from_value::<super::json_schema::JsonSchema>(
                serde_json::Value::Object(
                    schema.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
                ),
            ) {
                Ok(js) => {
                    let value = js.generate_from_rng(rng);
                    Some(RichContent::Text(serde_json::to_string(&value).unwrap_or_else(|_| "{}".into())))
                }
                Err(_) => Some(RichContent::Text("{}".into())),
            }
        }
        _ => {
            // Text or None — generate a random string.
            Some(RichContent::Text(random_string(rng, 10, 100)))
        }
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
            // Build a JSON object from the MCP tool's input_schema.
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
        Some(ResolvedTool::ResponseFormat) => None,
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
