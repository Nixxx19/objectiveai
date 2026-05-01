use std::time::Duration;

use futures::StreamExt;

use objectiveai::agent::completions::message::RichContent;
use objectiveai::agent::completions::request::{
    AgentCompletionCreateParams, ResponseFormat,
    ResponseFormatParam,
};
use objectiveai::agent::completions::response::streaming::{
    AgentCompletionChunk, AssistantResponseChunk, MessageChunk,
};
use objectiveai::agent::completions::response::unary::AgentCompletion;
use objectiveai::agent::mock::{Agent, AgentBase};

use crate::agent::completions::upstream_client::UpstreamClient;
use crate::test_mcp_server::{self, TestTool};

fn default_agent() -> Agent {
    Agent::try_from(AgentBase::default()).unwrap()
}

fn agent_with_top_logprobs(n: u64) -> Agent {
    Agent::try_from(AgentBase {
        top_logprobs: Some(n),
        ..Default::default()
    })
    .unwrap()
}

fn default_params_with_seed(seed: i64) -> AgentCompletionCreateParams {
    AgentCompletionCreateParams {
        messages: vec![],
        agent: objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(
            objectiveai::agent::InlineAgentBaseWithFallbacks {
                inner: objectiveai::agent::InlineAgentBase::Mock(AgentBase::default()),
                fallbacks: None,
            },
        ),
        provider: None,
        response_format: None,
        seed: Some(seed),
        stream: None,
        continuation: None,
    }
}

fn params_with_response_format(seed: i64, rf: ResponseFormat) -> AgentCompletionCreateParams {
    AgentCompletionCreateParams {
        response_format: Some(ResponseFormatParam::Single(rf)),
        ..default_params_with_seed(seed)
    }
}

fn default_client() -> std::sync::Arc<super::Client> {
    crate::test_clients::mock_upstream()
}

/// Runs the mock client to completion, accumulates all chunks, and returns AgentCompletion.
async fn run_mock(
    agent: &Agent,
    params: &AgentCompletionCreateParams,
    mcp_connection: Option<objectiveai::mcp::Connection>,
) -> AgentCompletion {
    let client = default_client();
    let messages = vec![];

    let stream = match client
        .create(
            "mock-test-id",
            1000,
            agent,
            None,
            params,
            &messages,
            mcp_connection,
            None,
            None,
            rust_decimal::Decimal::ONE,
            true,
            None,
            None,
            None,
            None,
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
                // Assert no usage anywhere.
                assert!(chunk.usage.is_none(), "chunk should not have usage");
                for msg in &chunk.messages {
                    if let MessageChunk::Assistant(asst) = msg {
                        assert!(asst.usage.is_none(), "assistant response chunk should not have usage");
                    }
                }
                // Assert exactly one assistant response chunk with index 0.
                assert_eq!(chunk.messages.len(), 1, "chunk should have exactly 1 message");
                match &chunk.messages[0] {
                    MessageChunk::Assistant(asst) => {
                        assert_eq!(asst.index, 0, "assistant response chunk index should be 0");
                    }
                    other => panic!("expected Assistant message chunk, got {other:?}"),
                }
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

// ---------------------------------------------------------------------------
// Snapshot helpers
// ---------------------------------------------------------------------------

fn normalize(mut c: AgentCompletion) -> AgentCompletion {
    c.normalize_for_tests();
    c
}

fn assert_snapshot(json: &str, path: &str, expected: &str) {
    crate::stream_harness::assert_snapshot(
        json, path, expected,
        "UPDATE_AGENT_COMPLETIONS_MOCK_CLIENT_TESTS_SNAPSHOTS",
    );
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_no_tools_no_response_format_seed_42() {
    let _permit = crate::test_clients::acquire_test_permit().await;
    let completion = normalize(run_mock(
        &default_agent(),
        &default_params_with_seed(42),
        None,
    ).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/mock/client_tests/test_no_tools_no_response_format_seed_42.json"),
        include_str!("../../../../assets/agent/completions/mock/client_tests/test_no_tools_no_response_format_seed_42.json"),
    );
}

#[tokio::test]
async fn test_no_tools_no_response_format_seed_123() {
    let _permit = crate::test_clients::acquire_test_permit().await;
    let completion = normalize(run_mock(
        &default_agent(),
        &default_params_with_seed(123),
        None,
    ).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/mock/client_tests/test_no_tools_no_response_format_seed_123.json"),
        include_str!("../../../../assets/agent/completions/mock/client_tests/test_no_tools_no_response_format_seed_123.json"),
    );
}

#[tokio::test]
async fn test_no_tools_no_response_format_seed_1() {
    let _permit = crate::test_clients::acquire_test_permit().await;
    let completion = normalize(run_mock(
        &default_agent(),
        &default_params_with_seed(1),
        None,
    ).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/mock/client_tests/test_no_tools_no_response_format_seed_1.json"),
        include_str!("../../../../assets/agent/completions/mock/client_tests/test_no_tools_no_response_format_seed_1.json"),
    );
}

#[tokio::test]
async fn test_no_tools_no_response_format_seed_2() {
    let _permit = crate::test_clients::acquire_test_permit().await;
    let completion = normalize(run_mock(
        &default_agent(),
        &default_params_with_seed(2),
        None,
    ).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/mock/client_tests/test_no_tools_no_response_format_seed_2.json"),
        include_str!("../../../../assets/agent/completions/mock/client_tests/test_no_tools_no_response_format_seed_2.json"),
    );
}

#[tokio::test]
async fn test_deterministic_with_same_seed() {
    let _permit = crate::test_clients::acquire_test_permit().await;
    let agent = default_agent();
    let params = default_params_with_seed(123);
    let a = normalize(run_mock(&agent, &params, None).await);
    let b = normalize(run_mock(&agent, &params, None).await);
    assert_eq!(a, b);
}

#[tokio::test]
async fn test_different_seeds_differ() {
    let _permit = crate::test_clients::acquire_test_permit().await;
    let agent = default_agent();
    let a = normalize(run_mock(&agent, &default_params_with_seed(1), None).await);
    let b = normalize(run_mock(&agent, &default_params_with_seed(2), None).await);
    assert_ne!(a, b);
}

#[tokio::test]
async fn test_grammar_response_format_rejected() {
    let _permit = crate::test_clients::acquire_test_permit().await;
    let client = default_client();
    let agent = default_agent();
    let params = params_with_response_format(42, ResponseFormat::Grammar {
        grammar: "root ::= 'hello'".into(),
    });

    let result = client
        .create(
            "test", 1000, &agent, None, &params, &[], None, None, None,
            rust_decimal::Decimal::ONE, true, None, None, None, None,
        )
        .await;
    match result {
        Err(e) => assert_eq!(objectiveai::error::StatusError::status(&e), 400),
        Ok(_) => panic!("expected error"),
    }
}

#[tokio::test]
async fn test_python_response_format_rejected() {
    let _permit = crate::test_clients::acquire_test_permit().await;
    let client = default_client();
    let agent = default_agent();
    let params = params_with_response_format(42, ResponseFormat::Python);

    let result = client
        .create(
            "test", 1000, &agent, None, &params, &[], None, None, None,
            rust_decimal::Decimal::ONE, true, None, None, None, None,
        )
        .await;
    match result {
        Err(e) => assert_eq!(objectiveai::error::StatusError::status(&e), 400),
        Ok(_) => panic!("expected error"),
    }
}

#[tokio::test]
async fn test_json_object_response_format() {
    let _permit = crate::test_clients::acquire_test_permit().await;
    let completion = normalize(run_mock(
        &default_agent(),
        &params_with_response_format(42, ResponseFormat::JsonObject),
        None,
    ).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/mock/client_tests/test_json_object_response_format.json"),
        include_str!("../../../../assets/agent/completions/mock/client_tests/test_json_object_response_format.json"),
    );
}

#[tokio::test]
async fn test_json_schema_response_format() {
    let _permit = crate::test_clients::acquire_test_permit().await;
    let params = params_with_response_format(42, ResponseFormat::JsonSchema {
        schema: indexmap::indexmap! {
            "type".into() => serde_json::json!("object"),
            "properties".into() => serde_json::json!({
                "name": {"type": "string"},
            }),
        },
    });
    let completion = normalize(run_mock(
        &default_agent(),
        &params,
        None,
    ).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/mock/client_tests/test_json_schema_response_format.json"),
        include_str!("../../../../assets/agent/completions/mock/client_tests/test_json_schema_response_format.json"),
    );
}

#[tokio::test]
async fn test_text_response_format() {
    let _permit = crate::test_clients::acquire_test_permit().await;
    let completion = normalize(run_mock(
        &default_agent(),
        &params_with_response_format(77, ResponseFormat::Text),
        None,
    ).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/mock/client_tests/test_text_response_format.json"),
        include_str!("../../../../assets/agent/completions/mock/client_tests/test_text_response_format.json"),
    );
}

#[tokio::test]
async fn test_with_mcp_tools() {
    let _permit = crate::test_clients::acquire_test_permit().await;
    let server = test_mcp_server::spawn("test", vec![TestTool::noop(make_mcp_tool(
        "search",
        Some(indexmap::indexmap! {
            "query".into() => serde_json::json!({"type": "string"}),
        }),
    ))]).await;
    let conn = test_mcp_server::connect_through_proxy(&[&server]).await;

    let completion = normalize(run_mock(
        &default_agent(),
        &default_params_with_seed(99),
        Some(conn),
    ).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/mock/client_tests/test_with_mcp_tools.json"),
        include_str!("../../../../assets/agent/completions/mock/client_tests/test_with_mcp_tools.json"),
    );
}

#[tokio::test]
async fn test_required_tool_call() {
    let _permit = crate::test_clients::acquire_test_permit().await;
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
    let params = params_with_response_format(42, rf.clone());

    let completion = normalize(run_mock(
        &default_agent(),
        &params,
        None,
    ).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/mock/client_tests/test_required_tool_call.json"),
        include_str!("../../../../assets/agent/completions/mock/client_tests/test_required_tool_call.json"),
    );
}

fn make_invention_tool(
    name: &'static str,
    schema: indexmap::IndexMap<String, serde_json::Value>,
) -> objectiveai::functions::inventions::InventionTool {
    objectiveai::functions::inventions::InventionTool {
        name: name.to_string(),
        description: "test",
        parameters: schema,
        call: std::sync::Arc::new(|_| Box::pin(async { Ok("ok".into()) })),
    }
}

fn make_mcp_tool(name: &str, properties: Option<indexmap::IndexMap<String, serde_json::Value>>) -> objectiveai::mcp::tool::Tool {
    objectiveai::mcp::tool::Tool {
        name: name.into(),
        title: None,
        description: Some(format!("{name} tool")),
        icons: None,
        input_schema: objectiveai::mcp::tool::ToolSchemaObject {
            r#type: objectiveai::mcp::tool::ToolSchemaType::Object,
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
    let _permit = crate::test_clients::acquire_test_permit().await;
    let server_a = test_mcp_server::spawn("test", vec![
        TestTool::noop(make_mcp_tool("get_forecast", Some(indexmap::indexmap! {
            "city".into() => serde_json::json!({"type": "string"}),
        }))),
        TestTool::noop(make_mcp_tool("get_alerts", None)),
    ]).await;
    let server_b = test_mcp_server::spawn("test", vec![
        TestTool::noop(make_mcp_tool("directions", Some(indexmap::indexmap! {
            "from".into() => serde_json::json!({"type": "string"}),
            "to".into() => serde_json::json!({"type": "string"}),
        }))),
    ]).await;
    let conn = test_mcp_server::connect_through_proxy(&[&server_a, &server_b]).await;

    let completion = normalize(run_mock(
        &default_agent(),
        &default_params_with_seed(50),
        Some(conn),
    ).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/mock/client_tests/test_multiple_mcp_tools.json"),
        include_str!("../../../../assets/agent/completions/mock/client_tests/test_multiple_mcp_tools.json"),
    );
}

#[tokio::test]
async fn test_invention_tools_only() {
    let _permit = crate::test_clients::acquire_test_permit().await;
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
    let server = test_mcp_server::spawn("test", vec![
        TestTool::from_invention(inv1),
        TestTool::from_invention(inv2),
    ]).await;
    let conn = test_mcp_server::connect_through_proxy(&[&server]).await;

    let completion = normalize(run_mock(
        &default_agent(),
        &default_params_with_seed(88),
        Some(conn),
    ).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/mock/client_tests/test_invention_tools_only.json"),
        include_str!("../../../../assets/agent/completions/mock/client_tests/test_invention_tools_only.json"),
    );
}

#[tokio::test]
async fn test_mcp_and_invention_no_response_format() {
    let _permit = crate::test_clients::acquire_test_permit().await;
    let inv = make_invention_tool("validate", indexmap::indexmap! {
        "type".into() => serde_json::json!("object"),
        "properties".into() => serde_json::json!({
            "data": {"type": "string"},
        }),
    });
    let mcp_server = test_mcp_server::spawn("test", vec![
        TestTool::noop(make_mcp_tool("query_db", Some(indexmap::indexmap! {
            "sql".into() => serde_json::json!({"type": "string"}),
        }))),
        TestTool::noop(make_mcp_tool("list_tables", None)),
    ]).await;
    let inv_server = test_mcp_server::spawn("test", vec![TestTool::from_invention(inv)]).await;
    let conn = test_mcp_server::connect_through_proxy(&[&mcp_server, &inv_server]).await;

    let completion = normalize(run_mock(
        &default_agent(),
        &default_params_with_seed(150),
        Some(conn),
    ).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/mock/client_tests/test_mcp_and_invention_no_response_format.json"),
        include_str!("../../../../assets/agent/completions/mock/client_tests/test_mcp_and_invention_no_response_format.json"),
    );
}

#[tokio::test]
async fn test_mcp_invention_and_response_format() {
    let _permit = crate::test_clients::acquire_test_permit().await;
    let inv = make_invention_tool("calculate", indexmap::indexmap! {
        "type".into() => serde_json::json!("object"),
        "properties".into() => serde_json::json!({
            "expression": {"type": "string"},
        }),
    });
    let mcp_server = test_mcp_server::spawn("test", vec![
        TestTool::noop(make_mcp_tool("web_search", Some(indexmap::indexmap! {
            "query".into() => serde_json::json!({"type": "string"}),
            "max_results".into() => serde_json::json!({"type": "integer"}),
        }))),
    ]).await;
    let inv_server = test_mcp_server::spawn("test", vec![TestTool::from_invention(inv)]).await;
    let conn = test_mcp_server::connect_through_proxy(&[&mcp_server, &inv_server]).await;
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
    let params = params_with_response_format(200, rf);

    let completion = normalize(run_mock(
        &default_agent(),
        &params,
        Some(conn),
    ).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/mock/client_tests/test_mcp_invention_and_response_format.json"),
        include_str!("../../../../assets/agent/completions/mock/client_tests/test_mcp_invention_and_response_format.json"),
    );
}

// ---------------------------------------------------------------------------
// Logprobs helpers
// ---------------------------------------------------------------------------

/// Collects all AssistantResponseChunks from the stream (not aggregated).
async fn collect_assistant_chunks(
    agent: &Agent,
    params: &AgentCompletionCreateParams,
    mcp_connection: Option<objectiveai::mcp::Connection>,
) -> Vec<AssistantResponseChunk> {
    let client = default_client();
    let messages = vec![];

    let stream = client
        .create(
            "mock-test-id",
            1000,
            agent,
            None,
            params,
            &messages,
            mcp_connection,
            None,
            None,
            rust_decimal::Decimal::ONE,
            true,
            None,
            None,
            None,
            None,
        )
        .await
        .expect("create failed");

    let mut chunks = Vec::new();
    let mut stream: std::pin::Pin<Box<dyn futures::Stream<Item = _> + Send>> = stream;

    while let Some(item) = stream.next().await {
        if let crate::agent::completions::upstream_client::StreamItem::Chunk(chunk) = item {
            // Assert no usage anywhere.
            assert!(chunk.usage.is_none(), "chunk should not have usage");
            // Assert exactly one assistant response chunk with index 0.
            assert_eq!(chunk.messages.len(), 1, "chunk should have exactly 1 message");
            match &chunk.messages[0] {
                MessageChunk::Assistant(asst) => {
                    assert_eq!(asst.index, 0, "assistant response chunk index should be 0");
                    assert!(asst.usage.is_none(), "assistant response chunk should not have usage");
                    chunks.push(asst.clone());
                }
                other => panic!("expected Assistant message chunk, got {other:?}"),
            }
        }
    }

    chunks
}

/// For each chunk that has content, asserts that the logprobs tokens reconstruct
/// the content text. The content of each token must appear among the top_logprobs
/// alternatives (or as the main token itself).
fn assert_logprobs_reconstruct_content(chunks: &[AssistantResponseChunk]) {
    for (i, chunk) in chunks.iter().enumerate() {
        let content_text = match &chunk.content {
            Some(RichContent::Text(t)) => t.as_str(),
            _ => continue,
        };

        let logprobs = chunk
            .logprobs
            .as_ref()
            .unwrap_or_else(|| panic!("chunk {i} has content but no logprobs"));
        let content_logprobs = logprobs
            .content
            .as_ref()
            .unwrap_or_else(|| panic!("chunk {i} has logprobs but no content logprobs"));

        // Reconstruct text from logprob tokens.
        let reconstructed: String = content_logprobs.iter().map(|lp| lp.token.as_str()).collect();
        assert_eq!(
            reconstructed, content_text,
            "chunk {i}: logprob tokens don't reconstruct content.\n  tokens: {:?}\n  content: {content_text:?}",
            content_logprobs.iter().map(|lp| &lp.token).collect::<Vec<_>>(),
        );
    }
}

// ---------------------------------------------------------------------------
// Logprobs tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_logprobs_top_2_seed_42() {
    let _permit = crate::test_clients::acquire_test_permit().await;
    // top_logprobs=2 is the minimum that survives AgentBase::prepare()
    // (top_logprobs of 0 or 1 get normalized to None).
    let agent = agent_with_top_logprobs(2);
    let params = default_params_with_seed(42);

    let chunks = collect_assistant_chunks(&agent, &params, None).await;
    assert_logprobs_reconstruct_content(&chunks);

    // Each logprob should have at most 2 top_logprobs entries.
    for (i, chunk) in chunks.iter().enumerate() {
        if chunk.content.is_none() {
            continue;
        }
        let content_logprobs = chunk.logprobs.as_ref().unwrap().content.as_ref().unwrap();
        for (j, lp) in content_logprobs.iter().enumerate() {
            assert!(
                lp.top_logprobs.len() <= 2,
                "chunk {i} logprob {j}: expected at most 2 top_logprobs entries, got {}",
                lp.top_logprobs.len(),
            );
        }
    }

    // Snapshot
    let completion = normalize(run_mock(&agent, &params, None).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/mock/client_tests/test_logprobs_top_2_seed_42.json"),
        include_str!("../../../../assets/agent/completions/mock/client_tests/test_logprobs_top_2_seed_42.json"),
    );
}

#[tokio::test]
async fn test_logprobs_top_5_seed_42() {
    let _permit = crate::test_clients::acquire_test_permit().await;
    let agent = agent_with_top_logprobs(5);
    let params = default_params_with_seed(42);

    let chunks = collect_assistant_chunks(&agent, &params, None).await;
    assert_logprobs_reconstruct_content(&chunks);

    for (i, chunk) in chunks.iter().enumerate() {
        if chunk.content.is_none() {
            continue;
        }
        let content_logprobs = chunk.logprobs.as_ref().unwrap().content.as_ref().unwrap();
        for (j, lp) in content_logprobs.iter().enumerate() {
            assert!(
                lp.top_logprobs.len() <= 5,
                "chunk {i} logprob {j}: expected at most 5 top_logprobs entries, got {}",
                lp.top_logprobs.len(),
            );
        }
    }

    let completion = normalize(run_mock(&agent, &params, None).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/mock/client_tests/test_logprobs_top_5_seed_42.json"),
        include_str!("../../../../assets/agent/completions/mock/client_tests/test_logprobs_top_5_seed_42.json"),
    );
}

#[tokio::test]
async fn test_logprobs_top_20_seed_42() {
    let _permit = crate::test_clients::acquire_test_permit().await;
    let agent = agent_with_top_logprobs(20);
    let params = default_params_with_seed(42);

    let chunks = collect_assistant_chunks(&agent, &params, None).await;
    assert_logprobs_reconstruct_content(&chunks);

    for (i, chunk) in chunks.iter().enumerate() {
        if chunk.content.is_none() {
            continue;
        }
        let content_logprobs = chunk.logprobs.as_ref().unwrap().content.as_ref().unwrap();
        for (j, lp) in content_logprobs.iter().enumerate() {
            assert!(
                lp.top_logprobs.len() <= 20,
                "chunk {i} logprob {j}: expected at most 20 top_logprobs entries, got {}",
                lp.top_logprobs.len(),
            );
        }
    }

    let completion = normalize(run_mock(&agent, &params, None).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/mock/client_tests/test_logprobs_top_20_seed_42.json"),
        include_str!("../../../../assets/agent/completions/mock/client_tests/test_logprobs_top_20_seed_42.json"),
    );
}

#[tokio::test]
async fn test_logprobs_top_3_json_object() {
    let _permit = crate::test_clients::acquire_test_permit().await;
    let agent = agent_with_top_logprobs(3);
    let params = params_with_response_format(42, ResponseFormat::JsonObject);

    let chunks = collect_assistant_chunks(&agent, &params, None).await;
    assert_logprobs_reconstruct_content(&chunks);

    let content_chunks: Vec<_> = chunks.iter().filter(|c| c.content.is_some()).collect();
    assert!(!content_chunks.is_empty(), "expected at least one content chunk");

    let full_content: String = content_chunks
        .iter()
        .filter_map(|c| match &c.content {
            Some(RichContent::Text(t)) => Some(t.as_str()),
            _ => None,
        })
        .collect();
    serde_json::from_str::<serde_json::Value>(&full_content)
        .expect("aggregated json_object content should be valid JSON");

    let completion = normalize(run_mock(&agent, &params, None).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/mock/client_tests/test_logprobs_top_3_json_object.json"),
        include_str!("../../../../assets/agent/completions/mock/client_tests/test_logprobs_top_3_json_object.json"),
    );
}

#[tokio::test]
async fn test_logprobs_top_10_json_schema() {
    let _permit = crate::test_clients::acquire_test_permit().await;
    let params = params_with_response_format(55, ResponseFormat::JsonSchema {
        schema: indexmap::indexmap! {
            "type".into() => serde_json::json!("object"),
            "properties".into() => serde_json::json!({
                "score": {"type": "number"},
                "label": {"type": "string"},
            }),
        },
    });
    let agent = agent_with_top_logprobs(10);

    let chunks = collect_assistant_chunks(&agent, &params, None).await;
    assert_logprobs_reconstruct_content(&chunks);

    let full_content: String = chunks
        .iter()
        .filter_map(|c| match &c.content {
            Some(RichContent::Text(t)) => Some(t.as_str()),
            _ => None,
        })
        .collect();
    let parsed: serde_json::Value = serde_json::from_str(&full_content)
        .expect("json_schema content should be valid JSON");
    assert!(parsed.get("score").is_some(), "expected 'score' key");
    assert!(parsed.get("label").is_some(), "expected 'label' key");

    let completion = normalize(run_mock(&agent, &params, None).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/mock/client_tests/test_logprobs_top_10_json_schema.json"),
        include_str!("../../../../assets/agent/completions/mock/client_tests/test_logprobs_top_10_json_schema.json"),
    );
}

#[tokio::test]
async fn test_logprobs_top_5_mcp_tools_seed_99() {
    let _permit = crate::test_clients::acquire_test_permit().await;
    let agent = agent_with_top_logprobs(5);
    let params = default_params_with_seed(99);

    let server = test_mcp_server::spawn("test", vec![
        TestTool::noop(make_mcp_tool("fetch_data", Some(indexmap::indexmap! {
            "url".into() => serde_json::json!({"type": "string"}),
        }))),
    ]).await;
    let conn = test_mcp_server::connect_through_proxy(&[&server]).await;

    let chunks = collect_assistant_chunks(&agent, &params, Some(conn.clone())).await;

    // Content chunks must have logprobs; tool_call chunks must NOT.
    for (i, chunk) in chunks.iter().enumerate() {
        if chunk.content.is_some() {
            assert!(
                chunk.logprobs.is_some(),
                "chunk {i}: content chunk should have logprobs when top_logprobs is set",
            );
        }
        if chunk.tool_calls.is_some() {
            assert!(
                chunk.logprobs.is_none(),
                "chunk {i}: tool_call chunk should not have logprobs",
            );
        }
    }
    assert_logprobs_reconstruct_content(&chunks);

    let completion = normalize(run_mock(&agent, &params, Some(conn)).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/mock/client_tests/test_logprobs_top_5_mcp_tools_seed_99.json"),
        include_str!("../../../../assets/agent/completions/mock/client_tests/test_logprobs_top_5_mcp_tools_seed_99.json"),
    );
}

#[tokio::test]
async fn test_logprobs_top_7_required_tool_call() {
    let _permit = crate::test_clients::acquire_test_permit().await;
    let agent = agent_with_top_logprobs(7);
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
    let params = params_with_response_format(88, rf);

    let chunks = collect_assistant_chunks(&agent, &params, None).await;

    let has_content = chunks.iter().any(|c| c.content.is_some());
    assert!(!has_content, "required tool call should produce no content chunks");

    let tool_chunks: Vec<_> = chunks.iter().filter(|c| c.tool_calls.is_some()).collect();
    assert!(!tool_chunks.is_empty(), "expected at least one tool_call chunk");
    for (i, chunk) in tool_chunks.iter().enumerate() {
        assert!(
            chunk.logprobs.is_none(),
            "tool_call chunk {i} should not have logprobs",
        );
    }

    let completion = normalize(run_mock(&agent, &params, None).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/mock/client_tests/test_logprobs_top_7_required_tool_call.json"),
        include_str!("../../../../assets/agent/completions/mock/client_tests/test_logprobs_top_7_required_tool_call.json"),
    );
}

#[tokio::test]
async fn test_logprobs_top_15_text_seed_33() {
    let _permit = crate::test_clients::acquire_test_permit().await;
    let agent = agent_with_top_logprobs(15);
    let params = params_with_response_format(33, ResponseFormat::Text);

    let chunks = collect_assistant_chunks(&agent, &params, None).await;
    assert_logprobs_reconstruct_content(&chunks);

    for chunk in &chunks {
        if let Some(lps) = &chunk.logprobs {
            for lp in lps.content.as_deref().unwrap_or_default() {
                assert_eq!(
                    lp.bytes.as_deref(),
                    Some(lp.token.as_bytes()),
                    "logprob bytes should match token string bytes",
                );
                for tlp in &lp.top_logprobs {
                    assert_eq!(
                        tlp.bytes.as_deref(),
                        Some(tlp.token.as_bytes()),
                        "top_logprob bytes should match token string bytes",
                    );
                }
            }
        }
    }

    let completion = normalize(run_mock(&agent, &params, None).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/mock/client_tests/test_logprobs_top_15_text_seed_33.json"),
        include_str!("../../../../assets/agent/completions/mock/client_tests/test_logprobs_top_15_text_seed_33.json"),
    );
}

#[tokio::test]
async fn test_logprobs_top_4_invention_mcp_response_format() {
    let _permit = crate::test_clients::acquire_test_permit().await;
    let agent = agent_with_top_logprobs(4);

    let inv = make_invention_tool("summarize", indexmap::indexmap! {
        "type".into() => serde_json::json!("object"),
        "properties".into() => serde_json::json!({
            "text": {"type": "string"},
        }),
    });
    let mcp_server = test_mcp_server::spawn("test", vec![
        TestTool::noop(make_mcp_tool("lookup_item", Some(indexmap::indexmap! {
            "id".into() => serde_json::json!({"type": "integer"}),
        }))),
    ]).await;
    let inv_server = test_mcp_server::spawn("test", vec![TestTool::from_invention(inv)]).await;
    let conn = test_mcp_server::connect_through_proxy(&[&mcp_server, &inv_server]).await;

    let rf = ResponseFormat::ToolCall {
        name: "final_answer".into(),
        description: "Submit the final answer".into(),
        schema: indexmap::indexmap! {
            "type".into() => serde_json::json!("object"),
            "properties".into() => serde_json::json!({
                "result": {"type": "string"},
                "score": {"type": "number"},
            }),
        },
        required: None,
    };
    let params = params_with_response_format(150, rf);

    let chunks = collect_assistant_chunks(&agent, &params, Some(conn.clone())).await;

    for (i, chunk) in chunks.iter().enumerate() {
        if chunk.content.is_some() {
            assert!(chunk.logprobs.is_some(), "chunk {i}: content without logprobs");
        }
        if chunk.tool_calls.is_some() {
            assert!(chunk.logprobs.is_none(), "chunk {i}: tool_call with logprobs");
        }
    }
    assert_logprobs_reconstruct_content(&chunks);

    let completion = normalize(run_mock(&agent, &params, Some(conn)).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/mock/client_tests/test_logprobs_top_4_invention_mcp_response_format.json"),
        include_str!("../../../../assets/agent/completions/mock/client_tests/test_logprobs_top_4_invention_mcp_response_format.json"),
    );
}

#[tokio::test]
async fn test_tools_not_allowed_with_required_tool_call() {
    let _permit = crate::test_clients::acquire_test_permit().await;
    let client = default_client();
    let agent = default_agent();
    let params = params_with_response_format(42, ResponseFormat::ToolCall {
        name: "my_tool".into(),
        description: "a tool".into(),
        schema: indexmap::IndexMap::new(),
        required: Some(true),
    });

    let result = client
        .create(
            "test", 1000, &agent, None, &params, &[], None, None, None,
            rust_decimal::Decimal::ONE, false, None, None, None, None,
        )
        .await;
    match result {
        Err(super::Error::ToolsNotAllowedWithRequiredToolCall) => {}
        Err(e) => panic!("expected ToolsNotAllowedWithRequiredToolCall, got {e}"),
        Ok(_) => panic!("expected error"),
    }
}

#[tokio::test]
async fn test_tools_not_allowed_with_optional_tool_call_ok() {
    let _permit = crate::test_clients::acquire_test_permit().await;
    let client = default_client();
    let agent = default_agent();
    let params = params_with_response_format(42, ResponseFormat::ToolCall {
        name: "my_tool".into(),
        description: "a tool".into(),
        schema: indexmap::IndexMap::new(),
        required: None,
    });

    // Optional tool call should succeed even with tools_enabled = false.
    let result = client
        .create(
            "test", 1000, &agent, None, &params, &[], None, None, None,
            rust_decimal::Decimal::ONE, false, None, None, None, None,
        )
        .await;
    assert!(result.is_ok(), "optional tool call should succeed when tools disabled");
}

#[tokio::test]
async fn test_tools_not_allowed_no_tool_calls_generated() {
    let _permit = crate::test_clients::acquire_test_permit().await;
    let agent = default_agent();
    // Use a response-format ToolCall so the mock has a callable tool to choose
    // from; tools_enabled=false should still suppress generation.
    let params = params_with_response_format(42, ResponseFormat::ToolCall {
        name: "my_tool".into(),
        description: "test tool".into(),
        schema: indexmap::IndexMap::new(),
        required: None,
    });
    let client = default_client();

    let stream = client
        .create(
            "test", 1000, &agent, None, &params, &[], None, None, None,
            rust_decimal::Decimal::ONE, false, None, None, None, None,
        )
        .await
        .expect("create should succeed");

    let mut stream: std::pin::Pin<Box<dyn futures::Stream<Item = _> + Send>> = stream;
    while let Some(item) = stream.next().await {
        if let crate::agent::completions::upstream_client::StreamItem::Chunk(chunk) = item {
            for msg in &chunk.messages {
                if let MessageChunk::Assistant(asst) = msg {
                    assert!(
                        asst.tool_calls.is_none(),
                        "should not generate tool calls when tools_enabled = false"
                    );
                }
            }
        }
    }
}

#[tokio::test]
async fn test_invention_agent_without_invention_tools() {
    let _permit = crate::test_clients::acquire_test_permit().await;
    let client = default_client();
    let agent = Agent::try_from(AgentBase {
        mode: Some(objectiveai::agent::mock::Mode::Invention),
        ..Default::default()
    })
    .unwrap();
    let params = default_params_with_seed(42);

    let result = client
        .create(
            "test", 1000, &agent, None, &params, &[], None, None, None,
            rust_decimal::Decimal::ONE, true, None, None, None, None,
        )
        .await;
    match result {
        Err(super::Error::InventionAgentWithoutInventionTools) => {}
        Err(e) => panic!("expected InventionAgentWithoutInventionTools, got {e}"),
        Ok(_) => panic!("expected error"),
    }
}

#[tokio::test]
async fn test_invention_agent_with_invention_tools_ok() {
    let _permit = crate::test_clients::acquire_test_permit().await;
    let agent = Agent::try_from(AgentBase {
        mode: Some(objectiveai::agent::mock::Mode::Invention),
        ..Default::default()
    })
    .unwrap();
    let params = default_params_with_seed(42);
    // The mock's invention dispatch (`mock::invention::pick_invention_tool`)
    // matches against the proxy-prefixed tool names that the
    // `objectiveai-function-invention` server emits. To exercise the
    // step-0 essay path here, we mount a tool whose final
    // proxy-prefixed name matches what the dispatcher expects: the
    // upstream server is named `objectiveai-function-invention`, and the
    // bare tool name is `WriteEssay`, so the prefix is
    // `objectiveai-function-invention_WriteEssay`.
    let inv = make_invention_tool("WriteEssay", indexmap::indexmap! {
        "type".into() => serde_json::json!("object"),
        "properties".into() => serde_json::json!({
            "essay": {"type": "string"},
        }),
    });
    let server = test_mcp_server::spawn(
        "objectiveai-function-invention",
        vec![TestTool::from_invention(inv)],
    )
    .await;
    let conn = test_mcp_server::connect_through_proxy(&[&server]).await;

    // With invention tools provided through the proxy, should succeed.
    let client = default_client();
    let result = client
        .create(
            "test", 1000, &agent, None, &params, &[], Some(conn), None, None,
            rust_decimal::Decimal::ONE, true,
            Some(objectiveai::functions::inventions::prompts::StepPromptType::AlphaScalarLeafFunction),
            Some(0), Some(3), None,
        )
        .await;
    assert!(result.is_ok(), "invention agent with tools should succeed");
}
