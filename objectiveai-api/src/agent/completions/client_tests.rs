use std::sync::Arc;

use objectiveai::agent::completions::message::{
    Message, RichContent, SimpleContent, UserMessage, DeveloperMessage,
};
use objectiveai::agent::completions::request::{
    AgentCompletionCreateParams, ResponseFormat,
    ResponseFormatParam,
};
use objectiveai::agent::completions::response::unary::{AgentCompletion, Message as UnaryMessage};
use objectiveai::agent::mock::AgentBase as MockAgentBase;

use crate::agent::completions::StreamItem;
use crate::ctx;

// ---------------------------------------------------------------------------
// Client constructor — delegates to the process-wide shared client.
// ---------------------------------------------------------------------------

fn make_client() -> Arc<crate::test_clients::AgentClient> {
    crate::test_clients::agent()
}

fn make_ctx() -> ctx::Context<ctx::DefaultContextExt, impl crate::ctx::persistent_cache::PersistentCacheClient> {
    ctx::Context::new(
        Arc::new(ctx::DefaultContextExt),
        Arc::new(ctx::persistent_cache::default::DefaultPersistentCacheClient),
        rust_decimal::Decimal::ONE,
        false,
        &axum::http::HeaderMap::new(),
    )
}

// ---------------------------------------------------------------------------
// Snapshot helpers
// ---------------------------------------------------------------------------

fn check_created_and_upstream(
    expected_created: &std::cell::Cell<Option<u64>>,
    expected_upstream: &std::cell::Cell<Option<objectiveai::agent::Upstream>>,
    i: usize,
    chunk: &objectiveai::agent::completions::response::streaming::AgentCompletionChunk,
) {
    match expected_created.get() {
        None => expected_created.set(Some(chunk.created)),
        Some(exp) => assert_eq!(chunk.created, exp, "chunk {i} has created {}, expected {exp}", chunk.created),
    }
    match expected_upstream.get() {
        None => expected_upstream.set(Some(chunk.upstream)),
        Some(exp) => assert_eq!(chunk.upstream, exp, "chunk {i} has upstream {:?}, expected {:?}", chunk.upstream, exp),
    }
}

async fn run_and_check<S: 'static>(
    stream: impl futures::Stream<Item = StreamItem<S>> + Unpin,
) -> AgentCompletion {
    let expected_created = std::cell::Cell::new(None);
    let expected_upstream: std::cell::Cell<Option<objectiveai::agent::Upstream>> = std::cell::Cell::new(None);
    let agg = crate::stream_harness::consume_stream_items(
        stream,
        |item| match item {
            StreamItem::Chunk(c) => Some(c),
            StreamItem::State(_) => None,
        },
        |agg, c| agg.push(c),
        |i, chunk| {
            check_created_and_upstream(&expected_created, &expected_upstream, i, chunk);
            assert!(chunk.messages.len() <= 1, "chunk {i} has {} messages, expected at most 1", chunk.messages.len());
            assert!(chunk.usage.is_none(), "chunk {i} (non-final) has usage, expected None");
            assert!(chunk.continuation.is_none(), "chunk {i} (non-final) has continuation, expected None");
        },
        |i, chunk| {
            check_created_and_upstream(&expected_created, &expected_upstream, i, chunk);
            assert!(chunk.messages.len() <= 1, "chunk {i} has {} messages, expected at most 1", chunk.messages.len());
            assert!(chunk.usage.is_none(), "chunk {i} (non-final) has usage, expected None");
            assert!(chunk.continuation.is_none(), "chunk {i} (non-final) has continuation, expected None");
        },
        |i, chunk| {
            check_created_and_upstream(&expected_created, &expected_upstream, i, chunk);
            assert!(chunk.usage.is_some(), "final chunk {i} has no usage, expected Some");
            assert!(chunk.continuation.is_some(), "final chunk {i} has no continuation, expected Some");
        },
    ).await;
    AgentCompletion::from(agg)
}

fn normalize(mut c: AgentCompletion) -> AgentCompletion {
    c.normalize_for_tests();
    c
}

fn assert_snapshot(json: &str, path: &str, expected: &str) {
    crate::stream_harness::assert_snapshot(
        json, path, expected,
        "UPDATE_AGENT_COMPLETIONS_CLIENT_TESTS_SNAPSHOTS",
    );
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Default mock agent, no error.
#[tokio::test]
async fn test_basic_mock_agent_seed_42() {
    let client = make_client();
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(
            objectiveai::agent::InlineAgentBaseWithFallbacks {
                inner: objectiveai::agent::InlineAgentBase::Mock(MockAgentBase::default()),
                fallbacks: None,
            },
        ),
        provider: None,
        response_format: None,
        seed: Some(42),
        stream: None,
        continuation: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, vec![], indexmap::IndexMap::new(), None, false, None, None, None, None)
        .await
        .expect("create_streaming should succeed");

    let completion = normalize(run_and_check(Box::pin(stream)).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/client_tests/test_basic_mock_agent_seed_42.json"),
        include_str!("../../../assets/agent/completions/client_tests/test_basic_mock_agent_seed_42.json"),
    );
}

/// Default mock agent with seed 123.
#[tokio::test]
async fn test_basic_mock_agent_seed_123() {
    let client = make_client();
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(
            objectiveai::agent::InlineAgentBaseWithFallbacks {
                inner: objectiveai::agent::InlineAgentBase::Mock(MockAgentBase::default()),
                fallbacks: None,
            },
        ),
        provider: None,
        response_format: None,
        seed: Some(123),
        stream: None,
        continuation: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, vec![], indexmap::IndexMap::new(), None, false, None, None, None, None)
        .await
        .expect("create_streaming should succeed");

    let completion = normalize(run_and_check(Box::pin(stream)).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/client_tests/test_basic_mock_agent_seed_123.json"),
        include_str!("../../../assets/agent/completions/client_tests/test_basic_mock_agent_seed_123.json"),
    );
}

/// Same seed produces identical streams.
#[tokio::test]
async fn test_deterministic_with_same_seed() {
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(
            objectiveai::agent::InlineAgentBaseWithFallbacks {
                inner: objectiveai::agent::InlineAgentBase::Mock(MockAgentBase::default()),
                fallbacks: None,
            },
        ),
        provider: None,
        response_format: None,
        seed: Some(77),
        stream: None,
        continuation: None,
    });

    let client_a = make_client();
    let stream_a = client_a
        .create_streaming(make_ctx(), params.clone(), None, None, vec![], indexmap::IndexMap::new(), None, false, None, None, None, None)
        .await
        .unwrap();
    let completion_a = normalize(run_and_check(Box::pin(stream_a)).await);

    let client_b = make_client();
    let stream_b = client_b
        .create_streaming(make_ctx(), params, None, None, vec![], indexmap::IndexMap::new(), None, false, None, None, None, None)
        .await
        .unwrap();
    let completion_b = normalize(run_and_check(Box::pin(stream_b)).await);
    assert_eq!(completion_a, completion_b);

    let json_a = serde_json::to_string_pretty(&completion_a).unwrap();
    assert_snapshot(
        &json_a,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/client_tests/test_deterministic_with_same_seed.json"),
        include_str!("../../../assets/agent/completions/client_tests/test_deterministic_with_same_seed.json"),
    );
}

/// Different seeds produce different streams.
#[tokio::test]
async fn test_different_seeds_differ() {
    let params_a = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(
            objectiveai::agent::InlineAgentBaseWithFallbacks {
                inner: objectiveai::agent::InlineAgentBase::Mock(MockAgentBase::default()),
                fallbacks: None,
            },
        ),
        provider: None,
        response_format: None,
        seed: Some(1),
        stream: None,
        continuation: None,
    });
    let params_b = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(
            objectiveai::agent::InlineAgentBaseWithFallbacks {
                inner: objectiveai::agent::InlineAgentBase::Mock(MockAgentBase::default()),
                fallbacks: None,
            },
        ),
        provider: None,
        response_format: None,
        seed: Some(2),
        stream: None,
        continuation: None,
    });

    let client_a = make_client();
    let stream_a = client_a
        .create_streaming(make_ctx(), params_a, None, None, vec![], indexmap::IndexMap::new(), None, false, None, None, None, None)
        .await
        .unwrap();
    let completion_a = normalize(run_and_check(Box::pin(stream_a)).await);

    let client_b = make_client();
    let stream_b = client_b
        .create_streaming(make_ctx(), params_b, None, None, vec![], indexmap::IndexMap::new(), None, false, None, None, None, None)
        .await
        .unwrap();
    let completion_b = normalize(run_and_check(Box::pin(stream_b)).await);

    assert_ne!(completion_a, completion_b);

    let json_a = serde_json::to_string_pretty(&completion_a).unwrap();
    assert_snapshot(
        &json_a,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/client_tests/test_different_seeds_differ_a.json"),
        include_str!("../../../assets/agent/completions/client_tests/test_different_seeds_differ_a.json"),
    );

    let json_b = serde_json::to_string_pretty(&completion_b).unwrap();
    assert_snapshot(
        &json_b,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/client_tests/test_different_seeds_differ_b.json"),
        include_str!("../../../assets/agent/completions/client_tests/test_different_seeds_differ_b.json"),
    );
}

/// Mock agent with error=true should fail.
#[tokio::test]
async fn test_mock_agent_with_error() {
    let client = make_client();
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(
            objectiveai::agent::InlineAgentBaseWithFallbacks {
                inner: objectiveai::agent::InlineAgentBase::Mock(MockAgentBase {
                    error: Some(true),
                    ..Default::default()
                }),
                fallbacks: None,
            },
        ),
        provider: None,
        response_format: None,
        seed: Some(42),
        stream: None,
        continuation: None,
    });

    let result = client
        .create_streaming(make_ctx(), params, None, None, vec![], indexmap::IndexMap::new(), None, false, None, None, None, None)
        .await;
    assert!(result.is_err(), "error agent should fail");
}

/// Messages: single user message.
#[tokio::test]
async fn test_with_single_user_message() {
    let client = make_client();
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![Message::User(UserMessage {
            content: RichContent::Text("Hello, world!".into()),
            name: None,
        })],
        agent: objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(
            objectiveai::agent::InlineAgentBaseWithFallbacks {
                inner: objectiveai::agent::InlineAgentBase::Mock(MockAgentBase::default()),
                fallbacks: None,
            },
        ),
        provider: None,
        response_format: None,
        seed: Some(42),
        stream: None,
        continuation: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, vec![], indexmap::IndexMap::new(), None, false, None, None, None, None)
        .await
        .expect("should succeed with user message");

    let completion = normalize(run_and_check(Box::pin(stream)).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/client_tests/test_with_single_user_message.json"),
        include_str!("../../../assets/agent/completions/client_tests/test_with_single_user_message.json"),
    );
}

/// Messages: developer + user messages.
#[tokio::test]
async fn test_with_developer_and_user_messages() {
    let client = make_client();
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![
            Message::Developer(DeveloperMessage {
                content: SimpleContent::Text("You are a helpful assistant.".into()),
                name: None,
            }),
            Message::User(UserMessage {
                content: RichContent::Text("What is 2+2?".into()),
                name: None,
            }),
        ],
        agent: objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(
            objectiveai::agent::InlineAgentBaseWithFallbacks {
                inner: objectiveai::agent::InlineAgentBase::Mock(MockAgentBase::default()),
                fallbacks: None,
            },
        ),
        provider: None,
        response_format: None,
        seed: Some(99),
        stream: None,
        continuation: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, vec![], indexmap::IndexMap::new(), None, false, None, None, None, None)
        .await
        .expect("should succeed with developer+user messages");

    let completion = normalize(run_and_check(Box::pin(stream)).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/client_tests/test_with_developer_and_user_messages.json"),
        include_str!("../../../assets/agent/completions/client_tests/test_with_developer_and_user_messages.json"),
    );
}

/// Response format: JsonObject.
#[tokio::test]
async fn test_json_object_response_format() {
    let client = make_client();
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(
            objectiveai::agent::InlineAgentBaseWithFallbacks {
                inner: objectiveai::agent::InlineAgentBase::Mock(MockAgentBase::default()),
                fallbacks: None,
            },
        ),
        provider: None,
        response_format: Some(ResponseFormatParam::Single(ResponseFormat::JsonObject)),
        seed: Some(42),
        stream: None,
        continuation: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, vec![], indexmap::IndexMap::new(), None, false, None, None, None, None)
        .await
        .expect("JsonObject should succeed");

    let completion = normalize(run_and_check(Box::pin(stream)).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/client_tests/test_json_object_response_format.json"),
        include_str!("../../../assets/agent/completions/client_tests/test_json_object_response_format.json"),
    );
}

/// Response format: JsonSchema with object schema.
#[tokio::test]
async fn test_json_schema_response_format() {
    let client = make_client();
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(
            objectiveai::agent::InlineAgentBaseWithFallbacks {
                inner: objectiveai::agent::InlineAgentBase::Mock(MockAgentBase::default()),
                fallbacks: None,
            },
        ),
        provider: None,
        response_format: Some(ResponseFormatParam::Single(ResponseFormat::JsonSchema {
            schema: indexmap::indexmap! {
                "type".into() => serde_json::json!("object"),
                "properties".into() => serde_json::json!({
                    "name": {"type": "string"},
                    "age": {"type": "integer"},
                }),
            },
        })),
        seed: Some(42),
        stream: None,
        continuation: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, vec![], indexmap::IndexMap::new(), None, false, None, None, None, None)
        .await
        .expect("JsonSchema should succeed");

    let completion = normalize(run_and_check(Box::pin(stream)).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/client_tests/test_json_schema_response_format.json"),
        include_str!("../../../assets/agent/completions/client_tests/test_json_schema_response_format.json"),
    );
}

/// Response format: Text.
#[tokio::test]
async fn test_text_response_format() {
    let client = make_client();
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(
            objectiveai::agent::InlineAgentBaseWithFallbacks {
                inner: objectiveai::agent::InlineAgentBase::Mock(MockAgentBase::default()),
                fallbacks: None,
            },
        ),
        provider: None,
        response_format: Some(ResponseFormatParam::Single(ResponseFormat::Text)),
        seed: Some(77),
        stream: None,
        continuation: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, vec![], indexmap::IndexMap::new(), None, false, None, None, None, None)
        .await
        .expect("Text should succeed");

    let completion = normalize(run_and_check(Box::pin(stream)).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/client_tests/test_text_response_format.json"),
        include_str!("../../../assets/agent/completions/client_tests/test_text_response_format.json"),
    );
}

/// Response format: Grammar should be rejected by mock client.
#[tokio::test]
async fn test_grammar_response_format_rejected() {
    let client = make_client();
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(
            objectiveai::agent::InlineAgentBaseWithFallbacks {
                inner: objectiveai::agent::InlineAgentBase::Mock(MockAgentBase::default()),
                fallbacks: None,
            },
        ),
        provider: None,
        response_format: Some(ResponseFormatParam::Single(ResponseFormat::Grammar {
            grammar: "root ::= 'hello'".into(),
        })),
        seed: Some(42),
        stream: None,
        continuation: None,
    });

    let result = client
        .create_streaming(make_ctx(), params, None, None, vec![], indexmap::IndexMap::new(), None, false, None, None, None, None)
        .await;
    assert!(result.is_err(), "Grammar should be rejected");
}

/// Response format: Python should be rejected by mock client.
#[tokio::test]
async fn test_python_response_format_rejected() {
    let client = make_client();
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(
            objectiveai::agent::InlineAgentBaseWithFallbacks {
                inner: objectiveai::agent::InlineAgentBase::Mock(MockAgentBase::default()),
                fallbacks: None,
            },
        ),
        provider: None,
        response_format: Some(ResponseFormatParam::Single(ResponseFormat::Python)),
        seed: Some(42),
        stream: None,
        continuation: None,
    });

    let result = client
        .create_streaming(make_ctx(), params, None, None, vec![], indexmap::IndexMap::new(), None, false, None, None, None, None)
        .await;
    assert!(result.is_err(), "Python should be rejected");
}

/// Response format: ToolCall with required=true.
#[tokio::test]
async fn test_required_tool_call_response_format() {
    let client = make_client();
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(
            objectiveai::agent::InlineAgentBaseWithFallbacks {
                inner: objectiveai::agent::InlineAgentBase::Mock(MockAgentBase::default()),
                fallbacks: None,
            },
        ),
        provider: None,
        response_format: Some(ResponseFormatParam::Single(ResponseFormat::ToolCall {
            name: "submit".into(),
            description: "Submit output".into(),
            schema: indexmap::indexmap! {
                "type".into() => serde_json::json!("object"),
                "properties".into() => serde_json::json!({
                    "answer": {"type": "string"},
                }),
            },
            required: Some(true),
        })),
        seed: Some(42),
        stream: None,
        continuation: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, vec![], indexmap::IndexMap::new(), None, false, None, None, None, None)
        .await
        .expect("required ToolCall should succeed");

    let completion = normalize(run_and_check(Box::pin(stream)).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/client_tests/test_required_tool_call_response_format.json"),
        include_str!("../../../assets/agent/completions/client_tests/test_required_tool_call_response_format.json"),
    );
}

/// Response format: ToolCall with required=None (optional).
#[tokio::test]
async fn test_optional_tool_call_response_format() {
    let client = make_client();
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(
            objectiveai::agent::InlineAgentBaseWithFallbacks {
                inner: objectiveai::agent::InlineAgentBase::Mock(MockAgentBase::default()),
                fallbacks: None,
            },
        ),
        provider: None,
        response_format: Some(ResponseFormatParam::Single(ResponseFormat::ToolCall {
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
        })),
        seed: Some(200),
        stream: None,
        continuation: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, vec![], indexmap::IndexMap::new(), None, false, None, None, None, None)
        .await
        .expect("optional ToolCall should succeed");

    let completion = normalize(run_and_check(Box::pin(stream)).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/client_tests/test_optional_tool_call_response_format.json"),
        include_str!("../../../assets/agent/completions/client_tests/test_optional_tool_call_response_format.json"),
    );
}

/// With invention tools provided.

/// With invention tools and ToolCall response format.

/// Single invention tool that returns an error.

/// Multiple user messages in a conversation.
#[tokio::test]
async fn test_multiple_user_messages() {
    let client = make_client();
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![
            Message::User(UserMessage {
                content: RichContent::Text("First message".into()),
                name: None,
            }),
            Message::User(UserMessage {
                content: RichContent::Text("Second message".into()),
                name: Some("alice".into()),
            }),
        ],
        agent: objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(
            objectiveai::agent::InlineAgentBaseWithFallbacks {
                inner: objectiveai::agent::InlineAgentBase::Mock(MockAgentBase::default()),
                fallbacks: None,
            },
        ),
        provider: None,
        response_format: None,
        seed: Some(55),
        stream: None,
        continuation: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, vec![], indexmap::IndexMap::new(), None, false, None, None, None, None)
        .await
        .expect("should succeed with multiple user messages");

    let completion = normalize(run_and_check(Box::pin(stream)).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/client_tests/test_multiple_user_messages.json"),
        include_str!("../../../assets/agent/completions/client_tests/test_multiple_user_messages.json"),
    );
}

/// Mock agent with error=Some(false) should succeed (normalized to None by prepare).
#[tokio::test]
async fn test_mock_agent_error_false_succeeds() {
    let client = make_client();
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(
            objectiveai::agent::InlineAgentBaseWithFallbacks {
                inner: objectiveai::agent::InlineAgentBase::Mock(MockAgentBase {
                    error: Some(false),
                    ..Default::default()
                }),
                fallbacks: None,
            },
        ),
        provider: None,
        response_format: None,
        seed: Some(42),
        stream: None,
        continuation: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, vec![], indexmap::IndexMap::new(), None, false, None, None, None, None)
        .await
        .expect("error=false should succeed");

    let completion = normalize(run_and_check(Box::pin(stream)).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/client_tests/test_mock_agent_error_false_succeeds.json"),
        include_str!("../../../assets/agent/completions/client_tests/test_mock_agent_error_false_succeeds.json"),
    );
}

/// Final stream item is always a Continuation::Mock.
#[tokio::test]
async fn test_final_item_is_mock_continuation() {
    let client = make_client();
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(
            objectiveai::agent::InlineAgentBaseWithFallbacks {
                inner: objectiveai::agent::InlineAgentBase::Mock(MockAgentBase::default()),
                fallbacks: None,
            },
        ),
        provider: None,
        response_format: None,
        seed: Some(42),
        stream: None,
        continuation: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, vec![], indexmap::IndexMap::new(), None, false, None, None, None, None)
        .await
        .unwrap();

    let completion = normalize(run_and_check(Box::pin(stream)).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/client_tests/test_final_item_is_mock_continuation.json"),
        include_str!("../../../assets/agent/completions/client_tests/test_final_item_is_mock_continuation.json"),
    );
}

/// PerAgent response format targeting the mock agent's ID.
#[tokio::test]
async fn test_per_agent_response_format() {
    let mock_base = MockAgentBase::default();
    let agent_id = mock_base.id();

    let client = make_client();
    let mut per_agent = indexmap::IndexMap::new();
    per_agent.insert(
        agent_id,
        ResponseFormat::JsonObject,
    );

    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(
            objectiveai::agent::InlineAgentBaseWithFallbacks {
                inner: objectiveai::agent::InlineAgentBase::Mock(mock_base),
                fallbacks: None,
            },
        ),
        provider: None,
        response_format: Some(ResponseFormatParam::PerAgent(per_agent)),
        seed: Some(42),
        stream: None,
        continuation: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, vec![], indexmap::IndexMap::new(), None, false, None, None, None, None)
        .await
        .expect("PerAgent response format should succeed");

    let completion = normalize(run_and_check(Box::pin(stream)).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/client_tests/test_per_agent_response_format.json"),
        include_str!("../../../assets/agent/completions/client_tests/test_per_agent_response_format.json"),
    );
}

/// PerAgent response format with unknown agent ID (should fall back to no format).
#[tokio::test]
async fn test_per_agent_response_format_unknown_id() {
    let client = make_client();
    let mut per_agent = indexmap::IndexMap::new();
    per_agent.insert(
        "nonexistent_agent_id_12345".into(),
        ResponseFormat::JsonObject,
    );

    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(
            objectiveai::agent::InlineAgentBaseWithFallbacks {
                inner: objectiveai::agent::InlineAgentBase::Mock(MockAgentBase::default()),
                fallbacks: None,
            },
        ),
        provider: None,
        response_format: Some(ResponseFormatParam::PerAgent(per_agent)),
        seed: Some(42),
        stream: None,
        continuation: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, vec![], indexmap::IndexMap::new(), None, false, None, None, None, None)
        .await
        .expect("PerAgent with unknown ID should succeed (no format applied)");

    let completion = normalize(run_and_check(Box::pin(stream)).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/client_tests/test_per_agent_response_format_unknown_id.json"),
        include_str!("../../../assets/agent/completions/client_tests/test_per_agent_response_format_unknown_id.json"),
    );
}

/// JsonSchema with nested object schema.
#[tokio::test]
async fn test_json_schema_nested_object() {
    let client = make_client();
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![Message::User(UserMessage {
            content: RichContent::Text("Generate a person".into()),
            name: None,
        })],
        agent: objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(
            objectiveai::agent::InlineAgentBaseWithFallbacks {
                inner: objectiveai::agent::InlineAgentBase::Mock(MockAgentBase::default()),
                fallbacks: None,
            },
        ),
        provider: None,
        response_format: Some(ResponseFormatParam::Single(ResponseFormat::JsonSchema {
            schema: indexmap::indexmap! {
                "type".into() => serde_json::json!("object"),
                "properties".into() => serde_json::json!({
                    "person": {
                        "type": "object",
                        "properties": {
                            "name": {"type": "string"},
                            "address": {
                                "type": "object",
                                "properties": {
                                    "street": {"type": "string"},
                                    "city": {"type": "string"},
                                }
                            }
                        }
                    }
                }),
            },
        })),
        seed: Some(99),
        stream: None,
        continuation: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, vec![], indexmap::IndexMap::new(), None, false, None, None, None, None)
        .await
        .expect("nested JsonSchema should succeed");

    let completion = normalize(run_and_check(Box::pin(stream)).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/client_tests/test_json_schema_nested_object.json"),
        include_str!("../../../assets/agent/completions/client_tests/test_json_schema_nested_object.json"),
    );
}

/// Fallback agents: primary errors, fallback succeeds.
#[tokio::test]
async fn test_fallback_agent_on_error() {
    let client = make_client();
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(
            objectiveai::agent::InlineAgentBaseWithFallbacks {
                inner: objectiveai::agent::InlineAgentBase::Mock(MockAgentBase {
                    error: Some(true),
                    ..Default::default()
                }),
                fallbacks: Some(vec![
                    objectiveai::agent::InlineAgentBase::Mock(MockAgentBase::default()),
                ]),
            },
        ),
        provider: None,
        response_format: None,
        seed: Some(42),
        stream: None,
        continuation: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, vec![], indexmap::IndexMap::new(), None, false, None, None, None, None)
        .await
        .expect("fallback agent should succeed when primary errors");

    let completion = normalize(run_and_check(Box::pin(stream)).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/client_tests/test_fallback_agent_on_error.json"),
        include_str!("../../../assets/agent/completions/client_tests/test_fallback_agent_on_error.json"),
    );
}

/// Both primary and fallback agents error — should fail.
#[tokio::test]
async fn test_all_agents_error() {
    let client = make_client();
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(
            objectiveai::agent::InlineAgentBaseWithFallbacks {
                inner: objectiveai::agent::InlineAgentBase::Mock(MockAgentBase {
                    error: Some(true),
                    ..Default::default()
                }),
                fallbacks: Some(vec![
                    objectiveai::agent::InlineAgentBase::Mock(MockAgentBase {
                        error: Some(true),
                        ..Default::default()
                    }),
                ]),
            },
        ),
        provider: None,
        response_format: None,
        seed: Some(42),
        stream: None,
        continuation: None,
    });

    let result = client
        .create_streaming(make_ctx(), params, None, None, vec![], indexmap::IndexMap::new(), None, false, None, None, None, None)
        .await;
    assert!(result.is_err(), "all agents erroring should fail");
}

/// Multiple fallback agents — first two error, third succeeds.
#[tokio::test]
async fn test_multiple_fallback_agents() {
    let client = make_client();
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(
            objectiveai::agent::InlineAgentBaseWithFallbacks {
                inner: objectiveai::agent::InlineAgentBase::Mock(MockAgentBase {
                    error: Some(true),
                    ..Default::default()
                }),
                fallbacks: Some(vec![
                    objectiveai::agent::InlineAgentBase::Mock(MockAgentBase {
                        error: Some(true),
                        ..Default::default()
                    }),
                    objectiveai::agent::InlineAgentBase::Mock(MockAgentBase::default()),
                ]),
            },
        ),
        provider: None,
        response_format: None,
        seed: Some(42),
        stream: None,
        continuation: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, vec![], indexmap::IndexMap::new(), None, false, None, None, None, None)
        .await
        .expect("third agent should succeed");

    let completion = normalize(run_and_check(Box::pin(stream)).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/client_tests/test_multiple_fallback_agents.json"),
        include_str!("../../../assets/agent/completions/client_tests/test_multiple_fallback_agents.json"),
    );
}

/// With continuation from a previous Mock run.
#[tokio::test]
async fn test_with_mock_continuation() {
    let _mock_agent = objectiveai::agent::mock::Agent::try_from(MockAgentBase::default()).unwrap();

    let client = make_client();
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(
            objectiveai::agent::InlineAgentBaseWithFallbacks {
                inner: objectiveai::agent::InlineAgentBase::Mock(MockAgentBase::default()),
                fallbacks: None,
            },
        ),
        provider: None,
        response_format: None,
        seed: Some(42),
        stream: None,
        continuation: None,
    });

    let continuation = crate::agent::completions::Continuation::Mock {
        items: vec![
            crate::agent::completions::ContinuationItem::State(
                objectiveai::agent::completions::message::AssistantMessage {
                    content: None, name: None, refusal: None, tool_calls: None, reasoning: None,
                },
            ),
        ],
        mcp_connection: None,
    };

    let stream = client
        .create_streaming(make_ctx(), params, Some(continuation), None, vec![], indexmap::IndexMap::new(), None, false, None, None, None, None)
        .await
        .expect("should succeed with continuation");

    let completion = normalize(run_and_check(Box::pin(stream)).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/client_tests/test_with_mock_continuation.json"),
        include_str!("../../../assets/agent/completions/client_tests/test_with_mock_continuation.json"),
    );
}

/// Stream produces chunks before the final state.
#[tokio::test]
async fn test_stream_yields_chunks_before_state() {
    let client = make_client();
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(
            objectiveai::agent::InlineAgentBaseWithFallbacks {
                inner: objectiveai::agent::InlineAgentBase::Mock(MockAgentBase::default()),
                fallbacks: None,
            },
        ),
        provider: None,
        response_format: None,
        seed: Some(42),
        stream: None,
        continuation: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, vec![], indexmap::IndexMap::new(), None, false, None, None, None, None)
        .await
        .unwrap();

    let completion = normalize(run_and_check(Box::pin(stream)).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/client_tests/test_stream_yields_chunks_before_state.json"),
        include_str!("../../../assets/agent/completions/client_tests/test_stream_yields_chunks_before_state.json"),
    );
}

/// Large seed value.
#[tokio::test]
async fn test_large_seed_value() {
    let client = make_client();
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(
            objectiveai::agent::InlineAgentBaseWithFallbacks {
                inner: objectiveai::agent::InlineAgentBase::Mock(MockAgentBase::default()),
                fallbacks: None,
            },
        ),
        provider: None,
        response_format: None,
        seed: Some(u64::MAX as i64),
        stream: None,
        continuation: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, vec![], indexmap::IndexMap::new(), None, false, None, None, None, None)
        .await
        .expect("large seed should succeed");

    let completion = normalize(run_and_check(Box::pin(stream)).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/client_tests/test_large_seed_value.json"),
        include_str!("../../../assets/agent/completions/client_tests/test_large_seed_value.json"),
    );
}

/// Seed 0.
#[tokio::test]
async fn test_seed_zero() {
    let client = make_client();
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(
            objectiveai::agent::InlineAgentBaseWithFallbacks {
                inner: objectiveai::agent::InlineAgentBase::Mock(MockAgentBase::default()),
                fallbacks: None,
            },
        ),
        provider: None,
        response_format: None,
        seed: Some(0),
        stream: None,
        continuation: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, vec![], indexmap::IndexMap::new(), None, false, None, None, None, None)
        .await
        .expect("seed 0 should succeed");

    let completion = normalize(run_and_check(Box::pin(stream)).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/client_tests/test_seed_zero.json"),
        include_str!("../../../assets/agent/completions/client_tests/test_seed_zero.json"),
    );
}

// ---------------------------------------------------------------------------
// Logprobs helpers
// ---------------------------------------------------------------------------

/// Asserts that every assistant message with content also has logprobs whose
/// tokens concatenate to reconstruct the content text.
fn assert_completion_logprobs(completion: &AgentCompletion) {
    for (i, msg) in completion.messages.iter().enumerate() {
        let asst = match msg {
            UnaryMessage::Assistant(a) => a,
            _ => continue,
        };
        let content = match &asst.content {
            Some(RichContent::Text(t)) => t.as_str(),
            _ => continue,
        };
        let logprobs = match &asst.logprobs {
            Some(lps) => lps,
            None => panic!("message {i}: assistant has content but no logprobs"),
        };
        let content_lps = match &logprobs.content {
            Some(lps) => lps,
            None => panic!("message {i}: logprobs present but content logprobs missing"),
        };
        let reconstructed: String = content_lps.iter().map(|lp| lp.token.as_str()).collect();
        assert_eq!(
            reconstructed, content,
            "message {i}: logprob tokens don't reconstruct content",
        );
    }
}

// ---------------------------------------------------------------------------
// Logprobs tests
// ---------------------------------------------------------------------------

/// Basic logprobs with plain text, no tools, no response format.
#[tokio::test]
async fn test_logprobs_basic_seed_42() {
    let client = make_client();
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![Message::User(UserMessage {
            content: RichContent::Text("Tell me something".into()),
            name: None,
        })],
        agent: objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(
            objectiveai::agent::InlineAgentBaseWithFallbacks {
                inner: objectiveai::agent::InlineAgentBase::Mock(MockAgentBase {
                    top_logprobs: Some(5),
                    ..Default::default()
                }),
                fallbacks: None,
            },
        ),
        provider: None,
        response_format: None,
        seed: Some(42),
        stream: None,
        continuation: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, vec![], indexmap::IndexMap::new(), None, false, None, None, None, None)
        .await
        .expect("logprobs basic should succeed");

    let completion = normalize(run_and_check(Box::pin(stream)).await);
    assert_completion_logprobs(&completion);

    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/client_tests/test_logprobs_basic_seed_42.json"),
        include_str!("../../../assets/agent/completions/client_tests/test_logprobs_basic_seed_42.json"),
    );
}

/// Logprobs with nested json_schema response format.
#[tokio::test]
async fn test_logprobs_json_schema_nested() {
    let client = make_client();
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(
            objectiveai::agent::InlineAgentBaseWithFallbacks {
                inner: objectiveai::agent::InlineAgentBase::Mock(MockAgentBase {
                    top_logprobs: Some(10),
                    ..Default::default()
                }),
                fallbacks: None,
            },
        ),
        provider: None,
        response_format: Some(ResponseFormatParam::Single(ResponseFormat::JsonSchema {
            schema: indexmap::indexmap! {
                "type".into() => serde_json::json!("object"),
                "properties".into() => serde_json::json!({
                    "result": {
                        "type": "object",
                        "properties": {
                            "label": {"type": "string"},
                            "values": {
                                "type": "array",
                                "items": {"type": "number"},
                            },
                        },
                    },
                }),
            },
        })),
        seed: Some(77),
        stream: None,
        continuation: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, vec![], indexmap::IndexMap::new(), None, false, None, None, None, None)
        .await
        .expect("logprobs json_schema nested should succeed");

    let completion = normalize(run_and_check(Box::pin(stream)).await);
    assert_completion_logprobs(&completion);

    // The content should parse as valid JSON.
    for msg in &completion.messages {
        if let UnaryMessage::Assistant(asst) = msg {
            if let Some(RichContent::Text(t)) = &asst.content {
                serde_json::from_str::<serde_json::Value>(t)
                    .expect("json_schema content should be valid JSON");
            }
        }
    }

    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/client_tests/test_logprobs_json_schema_nested.json"),
        include_str!("../../../assets/agent/completions/client_tests/test_logprobs_json_schema_nested.json"),
    );
}

/// Logprobs with invention tools — agent loop runs tool calls then content.

/// Logprobs survive through the continuation flow.
#[tokio::test]
async fn test_logprobs_with_continuation() {
    let mock_base = MockAgentBase {
        top_logprobs: Some(7),
        ..Default::default()
    };
    let _mock_agent = objectiveai::agent::mock::Agent::try_from(mock_base.clone()).unwrap();

    let client = make_client();
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(
            objectiveai::agent::InlineAgentBaseWithFallbacks {
                inner: objectiveai::agent::InlineAgentBase::Mock(mock_base),
                fallbacks: None,
            },
        ),
        provider: None,
        response_format: None,
        seed: Some(42),
        stream: None,
        continuation: None,
    });

    let continuation = crate::agent::completions::Continuation::Mock {
        items: vec![
            crate::agent::completions::ContinuationItem::State(
                objectiveai::agent::completions::message::AssistantMessage {
                    content: None, name: None, refusal: None, reasoning: None,
                    tool_calls: Some(vec![
                        objectiveai::agent::completions::message::AssistantToolCall::Function {
                            id: "1".into(),
                            function: objectiveai::agent::completions::message::AssistantToolCallFunction { name: "a".into(), arguments: String::new() },
                        },
                        objectiveai::agent::completions::message::AssistantToolCall::Function {
                            id: "2".into(),
                            function: objectiveai::agent::completions::message::AssistantToolCallFunction { name: "b".into(), arguments: String::new() },
                        },
                        objectiveai::agent::completions::message::AssistantToolCall::Function {
                            id: "3".into(),
                            function: objectiveai::agent::completions::message::AssistantToolCallFunction { name: "c".into(), arguments: String::new() },
                        },
                    ]),
                },
            ),
        ],
        mcp_connection: None,
    };

    let stream = client
        .create_streaming(make_ctx(), params, Some(continuation), None, vec![], indexmap::IndexMap::new(), None, false, None, None, None, None)
        .await
        .expect("logprobs with continuation should succeed");

    let completion = normalize(run_and_check(Box::pin(stream)).await);
    assert_completion_logprobs(&completion);

    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/client_tests/test_logprobs_with_continuation.json"),
        include_str!("../../../assets/agent/completions/client_tests/test_logprobs_with_continuation.json"),
    );
}

/// Primary agent errors, fallback agent has logprobs enabled.
#[tokio::test]
async fn test_logprobs_fallback_agent() {
    let client = make_client();
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(
            objectiveai::agent::InlineAgentBaseWithFallbacks {
                inner: objectiveai::agent::InlineAgentBase::Mock(MockAgentBase {
                    error: Some(true),
                    ..Default::default()
                }),
                fallbacks: Some(vec![
                    objectiveai::agent::InlineAgentBase::Mock(MockAgentBase {
                        top_logprobs: Some(12),
                        ..Default::default()
                    }),
                ]),
            },
        ),
        provider: None,
        response_format: None,
        seed: Some(55),
        stream: None,
        continuation: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, vec![], indexmap::IndexMap::new(), None, false, None, None, None, None)
        .await
        .expect("fallback with logprobs should succeed");

    let completion = normalize(run_and_check(Box::pin(stream)).await);
    assert_completion_logprobs(&completion);

    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/client_tests/test_logprobs_fallback_agent.json"),
        include_str!("../../../assets/agent/completions/client_tests/test_logprobs_fallback_agent.json"),
    );
}

/// Logprobs with PerAgent response format targeting mock agent's ID.
#[tokio::test]
async fn test_logprobs_per_agent_json_object() {
    let mock_base = MockAgentBase {
        top_logprobs: Some(4),
        ..Default::default()
    };
    let agent_id = mock_base.id();

    let client = make_client();
    let mut per_agent = indexmap::IndexMap::new();
    per_agent.insert(agent_id, ResponseFormat::JsonObject);

    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![Message::Developer(DeveloperMessage {
            content: SimpleContent::Text("Respond with JSON".into()),
            name: None,
        })],
        agent: objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(
            objectiveai::agent::InlineAgentBaseWithFallbacks {
                inner: objectiveai::agent::InlineAgentBase::Mock(mock_base),
                fallbacks: None,
            },
        ),
        provider: None,
        response_format: Some(ResponseFormatParam::PerAgent(per_agent)),
        seed: Some(33),
        stream: None,
        continuation: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, vec![], indexmap::IndexMap::new(), None, false, None, None, None, None)
        .await
        .expect("logprobs per-agent json_object should succeed");

    let completion = normalize(run_and_check(Box::pin(stream)).await);
    assert_completion_logprobs(&completion);

    // Content should be valid JSON.
    for msg in &completion.messages {
        if let UnaryMessage::Assistant(asst) = msg {
            if let Some(RichContent::Text(t)) = &asst.content {
                serde_json::from_str::<serde_json::Value>(t)
                    .expect("per-agent json_object content should be valid JSON");
            }
        }
    }

    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/client_tests/test_logprobs_per_agent_json_object.json"),
        include_str!("../../../assets/agent/completions/client_tests/test_logprobs_per_agent_json_object.json"),
    );
}

// ---------------------------------------------------------------------------
// Error probability tests (remote mock agent, invention tools, mid-stream error)
// ---------------------------------------------------------------------------

fn error_prob_50_remote() -> objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional {
    objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::Remote(
        objectiveai::RemotePathCommitOptional::Mock {
            name: "error-probability-50".into(),
        },
    )
}



