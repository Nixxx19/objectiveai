use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;

use objectiveai::agent::completions::message::{
    AssistantToolCall, AssistantToolCallFunction, RichContent,
};
use objectiveai::agent::completions::request::{
    AgentCompletionCreateParams, Agent as AgentParam, ResponseFormat,
    ResponseFormatParam,
};
use objectiveai::agent::completions::response::streaming::AgentCompletionChunk;
use objectiveai::agent::completions::response::unary::{
    AgentCompletion, AssistantResponse, Message, Object,
};
use objectiveai::agent::completions::response::{
    AssistantRole, FinishReason, Usage,
};
use objectiveai::agent::mock::{Agent, AgentBase};

use super::Client;
use crate::agent::completions::tool::{resolve_tools, ResolvedTool};
use crate::agent::completions::upstream_client::UpstreamClient;

fn default_agent() -> Agent {
    Agent::try_from(AgentBase::default()).unwrap()
}

fn default_params() -> AgentCompletionCreateParams {
    AgentCompletionCreateParams {
        messages: vec![],
        agent: AgentParam::Id("mock".into()),
        provider: None,
        agents: None,
        response_format: None,
        seed: None,
        stream: None,
        mcp_server_authorization: None,
    }
}

fn params_with_response_format(rf: ResponseFormat) -> AgentCompletionCreateParams {
    AgentCompletionCreateParams {
        response_format: Some(ResponseFormatParam::Single(rf)),
        ..default_params()
    }
}

/// Runs the mock client to completion, accumulates all chunks, and returns AgentCompletion.
async fn run_mock(
    seed: u64,
    agent: &Agent,
    params: &AgentCompletionCreateParams,
    tool_names: &[String],
    tool_map: &HashMap<String, ResolvedTool>,
) -> AgentCompletion {
    let client = Client {
        delay: Duration::ZERO,
        seed: Some(seed),
        max_tool_calls: None,
        tool_call_count: Arc::new(std::sync::atomic::AtomicU32::new(0)),
    };

    let messages = vec![];
    let mcp_connections: Vec<Arc<crate::mcp::Connection>> = vec![];

    let (stream, _state) = match client
        .create(
            "mock-test-id",
            1000,
            agent,
            params,
            &messages,
            &mcp_connections,
            None,
            tool_names,
            tool_map,
            None,
            None,
            rust_decimal::Decimal::ONE,
        )
        .await
    {
        Ok(v) => v,
        Err(e) => panic!("create failed: {e}"),
    };

    let mut accumulated: Option<AgentCompletionChunk> = None;
    let mut stream: std::pin::Pin<Box<dyn futures::Stream<Item = _> + Send>> = stream;

    while let Some(item) = stream.next().await {
        match item {
            crate::agent::completions::upstream_client::StreamItem::Chunk(chunk) => {
                match &mut accumulated {
                    Some(acc) => acc.push(&chunk),
                    None => accumulated = Some(chunk),
                }
            }
            crate::agent::completions::upstream_client::StreamItem::State(_) => {}
        }
    }

    AgentCompletion::from(accumulated.expect("should have received at least one chunk"))
}

/// Like `run_mock` but prints the result as JSON for discovering expected values.
#[allow(dead_code)]
async fn run_mock_print(
    seed: u64,
    agent: &Agent,
    params: &AgentCompletionCreateParams,
    tool_names: &[String],
    tool_map: &HashMap<String, ResolvedTool>,
) {
    let result = run_mock(seed, agent, params, tool_names, tool_map).await;
    println!(
        "{}",
        serde_json::to_string_pretty(&result).unwrap()
    );
}

const AGENT_ID: &str = "425mMcWqssOpPRrsDrjB0B";

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_no_tools_no_response_format_seed_42() {
    assert_eq!(
        run_mock(42, &default_agent(), &default_params(), &[], &HashMap::new()).await,
        AgentCompletion {
            id: "mock-test-id".into(),
            created: 1000,
            messages: vec![Message::Assistant(AssistantResponse {
                role: AssistantRole::Assistant,
                index: 0,
                created: 1000,
                agent: AGENT_ID.into(),
                model: "mock".into(),
                upstream_id: "mock-test-id".into(),
                reasoning: None,
                tool_calls: None,
                content: Some(RichContent::Text("pI2O z9cMAvUl1NiEaZ6LF2ySiYGZnOakGDdPOq1DF9FAdfwtCiYKmGzk".into())),
                refusal: None,
                finish_reason: FinishReason::Stop,
                logprobs: None,
                service_tier: None,
                system_fingerprint: None,
                provider: None,
            })],
            object: Object::AgentCompletion,
            usage: Usage { is_byok: false, ..Default::default() },
            upstream: objectiveai::agent::Upstream::Mock,
            error: None,
        }
    );
}

#[tokio::test]
async fn test_no_tools_no_response_format_seed_123() {
    assert_eq!(
        run_mock(123, &default_agent(), &default_params(), &[], &HashMap::new()).await,
        AgentCompletion {
            id: "mock-test-id".into(),
            created: 1000,
            messages: vec![Message::Assistant(AssistantResponse {
                role: AssistantRole::Assistant,
                index: 0,
                created: 1000,
                agent: AGENT_ID.into(),
                model: "mock".into(),
                upstream_id: "mock-test-id".into(),
                reasoning: Some("Uji6HAbOY Or iRLSM4pC 9Jd33ntOWLhJjv3GjYkMSwZbovScaOOAIkjewwzaRiMIK95oifDexUPwZ73c37SZv7LsTexSzLu6nHfjdC7b0sxBuo2A7PZKmO3zUejcXncZfBJY2oWU 4m7fIUq2Mn5KvKM4wAq7JI4qRJQamDINqvWhXIf1KI4kTWNx603elDyRaWONeGrox81Ns8tTlIUDgGbkBvZ6ymeCvPK3FJgSHbJZUhD3AjePYmerwaBRE60hflIiZtlnQhwMslPke8hpZaoKtURx3bWZmnPkytBcLPIhS6f0a7ZRcBjjl8cpX3WkomZYWHkcWBVAKbywOj611Ed mpXfcJ8Dw2QX7N8PFnOEP5lqeX2aKX7hPKkReZMRJFId2WfcMZp4nEzcZXb faaC05hTXVBECA1pJL8tBQ74gYytVqYROxiCIKi5BWSfmqDw1eMqIUijaRu73ypqqY9eNTCPz2ygx0VOsrXTzyDBmtKlyo02 lplw76K91F1UEj3uImU".into()),
                tool_calls: None,
                content: Some(RichContent::Text("8GtS63diXdQPtTY".into())),
                refusal: None,
                finish_reason: FinishReason::Stop,
                logprobs: None,
                service_tier: None,
                system_fingerprint: None,
                provider: None,
            })],
            object: Object::AgentCompletion,
            usage: Usage { is_byok: false, ..Default::default() },
            upstream: objectiveai::agent::Upstream::Mock,
            error: None,
        }
    );
}

#[tokio::test]
async fn test_no_tools_no_response_format_seed_1() {
    assert_eq!(
        run_mock(1, &default_agent(), &default_params(), &[], &HashMap::new()).await,
        AgentCompletion {
            id: "mock-test-id".into(),
            created: 1000,
            messages: vec![Message::Assistant(AssistantResponse {
                role: AssistantRole::Assistant,
                index: 0,
                created: 1000,
                agent: AGENT_ID.into(),
                model: "mock".into(),
                upstream_id: "mock-test-id".into(),
                reasoning: Some("YRmAnlWqG8Xy ShGjyImzdZ6YBjmwin4Fyt3ucEMtqp7NL8NchxqGlkMBw6Ddob8YqsdyOsmfCmk1X2qnRXlM0 hy123ELnd8REamSeq OZ82aiOY9HPkd5koH0XuvOcYueCCuiNik Y0VN80s8psAHeSnqQHLsZuqVF8UEPBKXZ69lhLF70ZSnhXcjC5XjiLpU36M tQRyxkyRNk1ZK1l5DBdQjncaoBdJO 8Z9tWbKqP93rajmrGuBVimXS9Y3vxngRoKUmyNRVB2hMhze3pV8n6j21bGPmp5RHJPMP79SPKSeExBHfaln O0QzGdzNr5eZ nLUKAABxhYSlzWaaYl0Wh4ykpow4qkGqagEOmVGhzx602SIIsvDv2e17JkhxrdEF9ff1mdIUx6QFjfcv6emzGUWPt8hGgg3TkSHIT0oBJtpi7JoderjPv96TH0KNPYv xodabe03LGICk6hqxviouzOTWyeeOseo8Zl0AxXmjoJaXFD3zniAJl0TfEsD0ckF5hOmG0jUIv1Esr4FdOo".into()),
                tool_calls: None,
                content: Some(RichContent::Text("txuEjGfXgzA4YXwShAItxBSRZ5gw6xdPxfBMPedTkbraOoZP5vrt4A6cNqjpRcaQt69JOcG2".into())),
                refusal: None,
                finish_reason: FinishReason::Stop,
                logprobs: None,
                service_tier: None,
                system_fingerprint: None,
                provider: None,
            })],
            object: Object::AgentCompletion,
            usage: Usage { is_byok: false, ..Default::default() },
            upstream: objectiveai::agent::Upstream::Mock,
            error: None,
        }
    );
}

#[tokio::test]
async fn test_no_tools_no_response_format_seed_2() {
    assert_eq!(
        run_mock(2, &default_agent(), &default_params(), &[], &HashMap::new()).await,
        AgentCompletion {
            id: "mock-test-id".into(),
            created: 1000,
            messages: vec![Message::Assistant(AssistantResponse {
                role: AssistantRole::Assistant,
                index: 0,
                created: 1000,
                agent: AGENT_ID.into(),
                model: "mock".into(),
                upstream_id: "mock-test-id".into(),
                reasoning: None,
                tool_calls: None,
                content: Some(RichContent::Text("6brDHFymQrG hXipGrTOtnfKP3NBxXlDLUh8r".into())),
                refusal: None,
                finish_reason: FinishReason::Stop,
                logprobs: None,
                service_tier: None,
                system_fingerprint: None,
                provider: None,
            })],
            object: Object::AgentCompletion,
            usage: Usage { is_byok: false, ..Default::default() },
            upstream: objectiveai::agent::Upstream::Mock,
            error: None,
        }
    );
}

#[tokio::test]
async fn test_deterministic_with_same_seed() {
    let agent = default_agent();
    let params = default_params();
    let a = run_mock(123, &agent, &params, &[], &HashMap::new()).await;
    let b = run_mock(123, &agent, &params, &[], &HashMap::new()).await;
    assert_eq!(a, b);
}

#[tokio::test]
async fn test_different_seeds_differ() {
    let agent = default_agent();
    let params = default_params();
    let a = run_mock(1, &agent, &params, &[], &HashMap::new()).await;
    let b = run_mock(2, &agent, &params, &[], &HashMap::new()).await;
    assert_ne!(a, b);
}

#[tokio::test]
async fn test_grammar_response_format_rejected() {
    let client = Client {
        delay: Duration::ZERO,
        seed: Some(42),
        max_tool_calls: None,
        tool_call_count: Arc::new(std::sync::atomic::AtomicU32::new(0)),
    };
    let agent = default_agent();
    let params = params_with_response_format(ResponseFormat::Grammar {
        grammar: "root ::= 'hello'".into(),
    });

    let result = client
        .create(
            "test", 1000, &agent, &params, &[], &[], None, &[],
            &HashMap::new(), None, None, rust_decimal::Decimal::ONE,
        )
        .await;
    match result {
        Err(e) => assert_eq!(e.code, 400),
        Ok(_) => panic!("expected error"),
    }
}

#[tokio::test]
async fn test_python_response_format_rejected() {
    let client = Client {
        delay: Duration::ZERO,
        seed: Some(42),
        max_tool_calls: None,
        tool_call_count: Arc::new(std::sync::atomic::AtomicU32::new(0)),
    };
    let agent = default_agent();
    let params = params_with_response_format(ResponseFormat::Python);

    let result = client
        .create(
            "test", 1000, &agent, &params, &[], &[], None, &[],
            &HashMap::new(), None, None, rust_decimal::Decimal::ONE,
        )
        .await;
    match result {
        Err(e) => assert_eq!(e.code, 400),
        Ok(_) => panic!("expected error"),
    }
}

#[tokio::test]
async fn test_json_object_response_format() {
    assert_eq!(
        run_mock(42, &default_agent(), &params_with_response_format(ResponseFormat::JsonObject), &[], &HashMap::new()).await,
        AgentCompletion {
            id: "mock-test-id".into(),
            created: 1000,
            messages: vec![Message::Assistant(AssistantResponse {
                role: AssistantRole::Assistant,
                index: 0,
                created: 1000,
                agent: AGENT_ID.into(),
                model: "mock".into(),
                upstream_id: "mock-test-id".into(),
                reasoning: None,
                tool_calls: None,
                content: Some(RichContent::Text("{}".into())),
                refusal: None,
                finish_reason: FinishReason::Stop,
                logprobs: None,
                service_tier: None,
                system_fingerprint: None,
                provider: None,
            })],
            object: Object::AgentCompletion,
            usage: Usage { is_byok: false, ..Default::default() },
            upstream: objectiveai::agent::Upstream::Mock,
            error: None,
        }
    );
}

#[tokio::test]
async fn test_json_schema_response_format() {
    let params = params_with_response_format(ResponseFormat::JsonSchema {
        schema: indexmap::indexmap! {
            "type".into() => serde_json::json!("object"),
            "properties".into() => serde_json::json!({
                "name": {"type": "string"},
            }),
        },
    });
    assert_eq!(
        run_mock(42, &default_agent(), &params, &[], &HashMap::new()).await,
        AgentCompletion {
            id: "mock-test-id".into(),
            created: 1000,
            messages: vec![Message::Assistant(AssistantResponse {
                role: AssistantRole::Assistant,
                index: 0,
                created: 1000,
                agent: AGENT_ID.into(),
                model: "mock".into(),
                upstream_id: "mock-test-id".into(),
                reasoning: None,
                tool_calls: None,
                content: Some(RichContent::Text("{\"name\":\"pH1N9z8cMzvTl0NiD\"}".into())),
                refusal: None,
                finish_reason: FinishReason::Stop,
                logprobs: None,
                service_tier: None,
                system_fingerprint: None,
                provider: None,
            })],
            object: Object::AgentCompletion,
            usage: Usage { is_byok: false, ..Default::default() },
            upstream: objectiveai::agent::Upstream::Mock,
            error: None,
        }
    );
}

#[tokio::test]
async fn test_text_response_format() {
    assert_eq!(
        run_mock(77, &default_agent(), &params_with_response_format(ResponseFormat::Text), &[], &HashMap::new()).await,
        AgentCompletion {
            id: "mock-test-id".into(),
            created: 1000,
            messages: vec![Message::Assistant(AssistantResponse {
                role: AssistantRole::Assistant,
                index: 0,
                created: 1000,
                agent: AGENT_ID.into(),
                model: "mock".into(),
                upstream_id: "mock-test-id".into(),
                reasoning: Some("YYIK4UQP8qBomWpwfdPAZlcZrkfkfeZEMW3qsBkZiugkhpW jcJ5gJ RsOvR8KGIQw51tOaxGZCac0OaLkXxY1snS7J5r3HRgLMJWVR411HtZFwzOu9bZXQt3 UClCLGsZp8zTlnoiopVl8r30piBuW1r7Vj1QDdn OMJWKWUjsl3UXJV7HglbTJ1zamEy5B2Hd2CvIDChNMjYttvRotpqS5HE4sO6b9bPe6GF9ds16imJk7".into()),
                tool_calls: None,
                content: Some(RichContent::Text("UyVAu16iCv7EwUJzHBxbrmp3zQzyeaqrOBzxGZktR4D86SpmLoqNDDAjqSLhVW Ra6llU8kF36c13KTl1bzPP4SIEAhrBImq7r".into())),
                refusal: None,
                finish_reason: FinishReason::Stop,
                logprobs: None,
                service_tier: None,
                system_fingerprint: None,
                provider: None,
            })],
            object: Object::AgentCompletion,
            usage: Usage { is_byok: false, ..Default::default() },
            upstream: objectiveai::agent::Upstream::Mock,
            error: None,
        }
    );
}

#[tokio::test]
async fn test_with_mcp_tools() {
    let conn = crate::mcp::Connection::new_for_test(
        "test-server".into(),
        "https://test.com/mcp".into(),
    );
    let tools = Arc::new(vec![crate::mcp::tool::Tool {
        name: "search".into(),
        title: None,
        description: Some("Search tool".into()),
        icons: None,
        input_schema: crate::mcp::tool::ToolSchema {
            r#type: crate::mcp::tool::ToolSchemaType::Object,
            properties: Some(indexmap::indexmap! {
                "query".into() => serde_json::json!({"type": "string"}),
            }),
            required: None,
            extra: indexmap::IndexMap::new(),
        },
        output_schema: None,
        annotations: None,
        execution: None,
        _meta: None,
    }]);

    let (tool_names, tool_map) = resolve_tools(&[conn], &[tools], None, None);
    assert_eq!(
        run_mock(99, &default_agent(), &default_params(), &tool_names, &tool_map).await,
        AgentCompletion {
            id: "mock-test-id".into(),
            created: 1000,
            messages: vec![Message::Assistant(AssistantResponse {
                role: AssistantRole::Assistant,
                index: 0,
                created: 1000,
                agent: AGENT_ID.into(),
                model: "mock".into(),
                upstream_id: "mock-test-id".into(),
                reasoning: Some("VRyoZ1i9ThhuvWUb4t1Q4U7i23zdbtCcLu2ALPenDcl7mSRfGVB5WSl078AWOK8eL z3dRbM7gXvuW i l8TbVOaE7JGSbOmzMKXTaM1BlNToN1rX2IZreCSmuq6u0m3VutDjTJa1Z6GHZf58rfGm 8YC6MYGiSM7igsqOBkrO5qbxrPs09HSgWobibQYR9KNkTZ8KsAaTcghQYPmzc37ibSCuIJ9MqHigwySU7URJPHMSuRXhXcxyKFdHkcUgOVwJp0xbjCwE6VfT0ZTqDt0ofQG2ejRxU8qnlyySIWeSKXN0Ln5 sNy1k9U1aTmag4ErUdIkiBLzaEy 0lhLDSxjcxThtBXwXjTc g28u4JdUTOJJKZKfYWGNlBLMvO2MHa5bFJUA65rPCNwTCcP9Qi".into()),
                tool_calls: None,
                content: Some(RichContent::Text("TWithyNuuDPlwE2jtgFfbWcyAwgVAcRVFImHEL".into())),
                refusal: None,
                finish_reason: FinishReason::Stop,
                logprobs: None,
                service_tier: None,
                system_fingerprint: None,
                provider: None,
            })],
            object: Object::AgentCompletion,
            usage: Usage { is_byok: false, ..Default::default() },
            upstream: objectiveai::agent::Upstream::Mock,
            error: None,
        }
    );
}

#[tokio::test]
async fn test_required_tool_call() {
    let rf = ResponseFormat::ToolCall {
        name: "submit".into(),
        description: "Submit output".into(),
        schema: indexmap::indexmap! {
            "type".into() => serde_json::json!("object"),
            "properties".into() => serde_json::json!({
                "answer": {"type": "string"},
            }),
        },
        required: Some(true),
    };
    let params = params_with_response_format(rf.clone());
    let (tool_names, tool_map) = resolve_tools(&[], &[], None, Some(&rf));

    assert_eq!(
        run_mock(42, &default_agent(), &params, &tool_names, &tool_map).await,
        AgentCompletion {
            id: "mock-test-id".into(),
            created: 1000,
            messages: vec![Message::Assistant(AssistantResponse {
                role: AssistantRole::Assistant,
                index: 0,
                created: 1000,
                agent: AGENT_ID.into(),
                model: "mock".into(),
                upstream_id: "mock-test-id".into(),
                reasoning: None,
                tool_calls: Some(vec![AssistantToolCall::Function {
                    id: "call_mock_15162404121733308702".into(),
                    function: AssistantToolCallFunction {
                        name: "submit".into(),
                        arguments: "{\"answer\":\"pH1N9z8cMzvTl0NiD\"}".into(),
                    },
                }]),
                content: None,
                refusal: None,
                finish_reason: FinishReason::ToolCalls,
                logprobs: None,
                service_tier: None,
                system_fingerprint: None,
                provider: None,
            })],
            object: Object::AgentCompletion,
            usage: Usage { is_byok: false, ..Default::default() },
            upstream: objectiveai::agent::Upstream::Mock,
            error: None,
        }
    );
}

fn make_invention_tool(
    name: &'static str,
    schema: indexmap::IndexMap<String, serde_json::Value>,
) -> objectiveai::functions::inventions::InventionTool {
    objectiveai::functions::inventions::InventionTool {
        name,
        description: "test",
        parameters: schema,
        call: std::sync::Arc::new(|_| Box::pin(async { Ok("ok".into()) })),
    }
}

fn make_mcp_tool(name: &str, properties: Option<indexmap::IndexMap<String, serde_json::Value>>) -> crate::mcp::tool::Tool {
    crate::mcp::tool::Tool {
        name: name.into(),
        title: None,
        description: Some(format!("{name} tool")),
        icons: None,
        input_schema: crate::mcp::tool::ToolSchema {
            r#type: crate::mcp::tool::ToolSchemaType::Object,
            properties,
            required: None,
            extra: indexmap::IndexMap::new(),
        },
        output_schema: None,
        annotations: None,
        execution: None,
        _meta: None,
    }
}

// --- Tests with diverse tool configurations ---

#[tokio::test]
async fn test_multiple_mcp_tools() {
    let conn1 = crate::mcp::Connection::new_for_test("weather".into(), "https://weather.com/mcp".into());
    let conn2 = crate::mcp::Connection::new_for_test("maps".into(), "https://maps.com/mcp".into());
    let tools1 = Arc::new(vec![
        make_mcp_tool("get_forecast", Some(indexmap::indexmap! {
            "city".into() => serde_json::json!({"type": "string"}),
        })),
        make_mcp_tool("get_alerts", None),
    ]);
    let tools2 = Arc::new(vec![
        make_mcp_tool("directions", Some(indexmap::indexmap! {
            "from".into() => serde_json::json!({"type": "string"}),
            "to".into() => serde_json::json!({"type": "string"}),
        })),
    ]);
    let (tool_names, tool_map) = resolve_tools(&[conn1, conn2], &[tools1, tools2], None, None);
    assert_eq!(
        run_mock(50, &default_agent(), &default_params(), &tool_names, &tool_map).await,
        AgentCompletion {
            id: "mock-test-id".into(),
            created: 1000,
            messages: vec![Message::Assistant(AssistantResponse {
                role: AssistantRole::Assistant,
                index: 0,
                created: 1000,
                agent: AGENT_ID.into(),
                model: "mock".into(),
                upstream_id: "mock-test-id".into(),
                reasoning: Some("6eQ3F7G WtO c1gbEHD4a5tOh6AC9JRXoBY06ZSBtBe9ZdLSIBdE 9lKtWBkODlLuHVGeUjosZW7iEy1EombDdr5LVjyIx9HCw6TYWfA9bI80SKeoxiZPaUJCI1IyjtRrtBXdrAeDE0xq4pr1lXvF8V2wmVmMm2ScQZ3JOMCATXt7h7badD6f vuITcBIMm7g9kQsW4oYO8O gUoB3Z82Dge4LB4SrIXIoDEnoKRzd8c1Q04bPwbL3UQCExVpKQFmahi4mzzyBCBBq09JNCBav7jhxtvPB81KfKF5qqbnxF4JSdrOWuPmI2eA1DQVlICJmekz0MJmyG".into()),
                tool_calls: Some(vec![
                    AssistantToolCall::Function {
                        id: "call_mock_12791539320035428755".into(),
                        function: AssistantToolCallFunction {
                            name: "directions".into(),
                            arguments: "{\"from\":\"uLGraSV4vlCYGs2CJbz17zTkmo\",\"to\":\"pgBRoC0hftsPsGEg5y9yciuq\"}".into(),
                        },
                    },
                    AssistantToolCall::Function {
                        id: "call_mock_9258486154652128339".into(),
                        function: AssistantToolCallFunction {
                            name: "directions".into(),
                            arguments: "{\"from\":\"ODyHavBAXY7ilU\",\"to\":\"N6NZw\"}".into(),
                        },
                    },
                ]),
                content: None,
                refusal: None,
                finish_reason: FinishReason::ToolCalls,
                logprobs: None,
                service_tier: None,
                system_fingerprint: None,
                provider: None,
            })],
            object: Object::AgentCompletion,
            usage: Usage { is_byok: false, ..Default::default() },
            upstream: objectiveai::agent::Upstream::Mock,
            error: None,
        }
    );
}

#[tokio::test]
async fn test_invention_tools_only() {
    let inv1 = make_invention_tool("execute_code", indexmap::indexmap! {
        "type".into() => serde_json::json!("object"),
        "properties".into() => serde_json::json!({
            "language": {"type": "string"},
            "code": {"type": "string"},
        }),
    });
    let inv2 = make_invention_tool("read_file", indexmap::indexmap! {
        "type".into() => serde_json::json!("object"),
        "properties".into() => serde_json::json!({
            "path": {"type": "string"},
        }),
    });
    let (tool_names, tool_map) = resolve_tools(&[], &[], Some(&[inv1, inv2]), None);
    assert_eq!(
        run_mock(88, &default_agent(), &default_params(), &tool_names, &tool_map).await,
        AgentCompletion {
            id: "mock-test-id".into(),
            created: 1000,
            messages: vec![Message::Assistant(AssistantResponse {
                role: AssistantRole::Assistant,
                index: 0,
                created: 1000,
                agent: AGENT_ID.into(),
                model: "mock".into(),
                upstream_id: "mock-test-id".into(),
                reasoning: Some("32vtUo3rxTcDjLHp3Eyst9aXBN7ocD3KQQKiVpJzrhLUL M213ea vGzRR0FBs3WexDDnzVdyRxXVdq8J2zwkMh5B8BbVV7jt3parzDmkK2ydNpOSESFLc9uoGKBNwfH4 bBWtK0HIg8CPGmRIqzqSuCmhQy kZQrQpak9doRcL8BQiCq5CIsfFDgIvgtXZ2i7c4Uct7nFEt R2zwvkSyxW9WHgswSOw7FFLFqe1ieaa37WKnh1fDvpKIqgLSn0P3GcWjQjKULRBA3NmDdDGUrTeK80hCUG21kI0CKR31H79tBhrLY571wc0eEYButlLL9bj".into()),
                tool_calls: Some(vec![
                    AssistantToolCall::Function {
                        id: "call_mock_3455269600397602557".into(),
                        function: AssistantToolCallFunction {
                            name: "execute_code".into(),
                            arguments: "{\"language\":\"tWoJ8xOQ\",\"code\":\"4Rp5lBZMaA4fSggDNTR\"}".into(),
                        },
                    },
                    AssistantToolCall::Function {
                        id: "call_mock_8479485837367955825".into(),
                        function: AssistantToolCallFunction {
                            name: "execute_code".into(),
                            arguments: "{\"language\":\"ARs2MzUkDNz1svAXf\",\"code\":\"uFllY5nk\"}".into(),
                        },
                    },
                    AssistantToolCall::Function {
                        id: "call_mock_9211892752638480659".into(),
                        function: AssistantToolCallFunction {
                            name: "execute_code".into(),
                            arguments: "{\"language\":\"7zV2sOFv0dkULWBuFQLGZ\",\"code\":\"ZypwpWFxLyCzoF7EJ7tR\"}".into(),
                        },
                    },
                ]),
                content: None,
                refusal: None,
                finish_reason: FinishReason::ToolCalls,
                logprobs: None,
                service_tier: None,
                system_fingerprint: None,
                provider: None,
            })],
            object: Object::AgentCompletion,
            usage: Usage { is_byok: false, ..Default::default() },
            upstream: objectiveai::agent::Upstream::Mock,
            error: None,
        }
    );
}

#[tokio::test]
async fn test_mcp_and_invention_no_response_format() {
    let conn = crate::mcp::Connection::new_for_test("db".into(), "https://db.com/mcp".into());
    let tools = Arc::new(vec![
        make_mcp_tool("query_db", Some(indexmap::indexmap! {
            "sql".into() => serde_json::json!({"type": "string"}),
        })),
        make_mcp_tool("list_tables", None),
    ]);
    let inv = make_invention_tool("validate", indexmap::indexmap! {
        "type".into() => serde_json::json!("object"),
        "properties".into() => serde_json::json!({
            "data": {"type": "string"},
        }),
    });
    let (tool_names, tool_map) = resolve_tools(&[conn], &[tools], Some(&[inv]), None);
    assert_eq!(
        run_mock(150, &default_agent(), &default_params(), &tool_names, &tool_map).await,
        AgentCompletion {
            id: "mock-test-id".into(),
            created: 1000,
            messages: vec![Message::Assistant(AssistantResponse {
                role: AssistantRole::Assistant,
                index: 0,
                created: 1000,
                agent: AGENT_ID.into(),
                model: "mock".into(),
                upstream_id: "mock-test-id".into(),
                reasoning: Some("EhOTnkSyywJYGzQK1SGD8tciSZUFdXWZGBaOM4faKR8TVl8ygudr9v3Foj3ljiCWz6d0tYzQc80kGYjKOsXfTlFQO58lgCade5r8tH3cKXhbAk7GgzsfA6o 1okzaJ8URf8cEypIlTDnRZl uJ7X6zM3Z3MtnzdrFx1blAxOiK6gP9rzcprhHLFlklXpYYUu8INpqcG2BvyUptGF euKpZA4g6SIphEjPojpS07l8W89cvyHNcwu7qNRZvz0ykHYP5i7 fyN0YMOurctyEy9sbEFRg9UUbfJgV  7QJqioZBUNItbJ7hDcj3LYXL36udBc8gpOe8wQFQwktzYVkW7NLo9wf16h6a8mTr6M7jk28ipIEt2D4SyqANJbrirYfrj8AksViecSXehq5s8MoAMduMdtCmNegVrR Uu1LeJ6o2GkPI846sMHSUfJ3gIgOPxW hvbAso HSzyBfN6fN43J4jqlZfJj8UYKo6ZFHTcs1 12ANkDyvEcPYwPjWMQNr7vclPjlKmY5UZDHl8ym9srpkLw3Qfnl5Gpz9vNZI9mKO4rdIQVa vUjMa1U5kCMSCoY23jyp9onS71W9ktebmy06atR4bJ28UKGUCnsjPLbnCQ8G5rl2z33yyW0q7NuQTuxaBZU6m3Mn7RSb4sog5qNYyKz9Lz3T5PJPwqHbRg5".into()),
                tool_calls: Some(vec![AssistantToolCall::Function {
                    id: "call_mock_11810568058892855581".into(),
                    function: AssistantToolCallFunction {
                        name: "list_tables".into(),
                        arguments: "{}".into(),
                    },
                }]),
                content: None,
                refusal: None,
                finish_reason: FinishReason::ToolCalls,
                logprobs: None,
                service_tier: None,
                system_fingerprint: None,
                provider: None,
            })],
            object: Object::AgentCompletion,
            usage: Usage { is_byok: false, ..Default::default() },
            upstream: objectiveai::agent::Upstream::Mock,
            error: None,
        }
    );
}

#[tokio::test]
async fn test_mcp_invention_and_response_format() {
    let conn = crate::mcp::Connection::new_for_test("search-api".into(), "https://search.com/mcp".into());
    let tools = Arc::new(vec![
        make_mcp_tool("web_search", Some(indexmap::indexmap! {
            "query".into() => serde_json::json!({"type": "string"}),
            "max_results".into() => serde_json::json!({"type": "integer"}),
        })),
    ]);
    let inv = make_invention_tool("calculate", indexmap::indexmap! {
        "type".into() => serde_json::json!("object"),
        "properties".into() => serde_json::json!({
            "expression": {"type": "string"},
        }),
    });
    let rf = ResponseFormat::ToolCall {
        name: "submit_answer".into(),
        description: "Submit the final answer".into(),
        schema: indexmap::indexmap! {
            "type".into() => serde_json::json!("object"),
            "properties".into() => serde_json::json!({
                "answer": {"type": "string"},
                "confidence": {"type": "number"},
            }),
        },
        required: None,
    };
    let params = params_with_response_format(rf.clone());
    let (tool_names, tool_map) = resolve_tools(&[conn], &[tools], Some(&[inv]), Some(&rf));
    assert_eq!(
        run_mock(200, &default_agent(), &params, &tool_names, &tool_map).await,
        AgentCompletion {
            id: "mock-test-id".into(),
            created: 1000,
            messages: vec![Message::Assistant(AssistantResponse {
                role: AssistantRole::Assistant,
                index: 0,
                created: 1000,
                agent: AGENT_ID.into(),
                model: "mock".into(),
                upstream_id: "mock-test-id".into(),
                reasoning: Some("gdNCbElZUfcQ96Lmg8o TDcs1FAJHGfvifoBkUeKdnyIdi3hfgJZNA56OQZLeIdOwdECGFbudMVMBMSJnyn6EHKw7il5yH5kednd70oXisicmfe tc4U5jFrMS6Y7MXe2RxYDjYoY3zkU9 F7txJ4Aoj0lNb5QeTJvlpOSHj3nGsl0WRnKmfixgIcuhRcMrrL1WZqYK2nFQYbiZJrqhR4anJ 5asHhnseqK7rstgZ7vjkAgEMgavAgjuVbQchqwgHT1qfmFiDX0YzeCY1PBcSGAU QiieXXod0OPtikXVCCaHsUHmriXscozoEuNDI57v 0UFM1As6ODAIdXH3wKc91XafPtCXmqDf3W1VmauFy0HLFwRALaQflskfbrAfB5SQbmjfjC8USX6IiHDKai3an2Dn3jJA3pAs stWVrwErmQ3dnBy26id06fCPXmqrpt4GN21mdW 3BYmtiMCNDjiHp9LmsHrzWXXobMIV5UcQleqXqHWrHz".into()),
                tool_calls: Some(vec![AssistantToolCall::Function {
                    id: "call_mock_15123337590676269645".into(),
                    function: AssistantToolCallFunction {
                        name: "web_search".into(),
                        arguments: "{\"query\":\"7GlSxjobUsZ7\",\"max_results\":81}".into(),
                    },
                }]),
                content: None,
                refusal: None,
                finish_reason: FinishReason::ToolCalls,
                logprobs: None,
                service_tier: None,
                system_fingerprint: None,
                provider: None,
            })],
            object: Object::AgentCompletion,
            usage: Usage { is_byok: false, ..Default::default() },
            upstream: objectiveai::agent::Upstream::Mock,
            error: None,
        }
    );
}
