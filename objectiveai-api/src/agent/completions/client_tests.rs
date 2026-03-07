use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;

use objectiveai::agent::completions::message::{
    Message, RichContent, SimpleContent, UserMessage, DeveloperMessage,
};
use objectiveai::agent::completions::request::{
    Agent as AgentParam, AgentCompletionCreateParams, ResponseFormat,
    ResponseFormatParam,
};
use objectiveai::agent::completions::response::unary::{AgentCompletion, Message as UnaryMessage};
use objectiveai::agent::mock::AgentBase as MockAgentBase;

use crate::agent::completions::upstream_client::UnimplementedUpstreamClient;
use crate::agent::completions::StreamItem;
use crate::ctx;

// ---------------------------------------------------------------------------
// Stub fetcher — never actually called since we always provide inline agents.
// ---------------------------------------------------------------------------

struct StubFetcher;

#[async_trait::async_trait]
impl crate::agent::fetcher::Fetcher<ctx::DefaultContextExt> for StubFetcher {
    async fn fetch(
        &self,
        _ctx: ctx::Context<ctx::DefaultContextExt>,
        _id: &str,
    ) -> Result<
        Option<(objectiveai::agent::Agent, u64)>,
        objectiveai::error::ResponseError,
    > {
        Err(objectiveai::error::ResponseError {
            code: 501,
            message: serde_json::json!("stub fetcher should not be called"),
        })
    }
}

struct StubUsageHandler;

impl crate::agent::completions::usage_handler::UsageHandler<ctx::DefaultContextExt>
    for StubUsageHandler
{
    fn handle_usage(
        &self,
        _ctx: ctx::Context<ctx::DefaultContextExt>,
        _request: Arc<objectiveai::agent::completions::request::AgentCompletionCreateParams>,
        _response: objectiveai::agent::completions::response::unary::AgentCompletion,
    ) -> impl std::future::Future<Output = ()> + Send + 'static {
        async {}
    }
}

// ---------------------------------------------------------------------------
// Client constructor
// ---------------------------------------------------------------------------

fn make_client() -> super::Client<
    ctx::DefaultContextExt,
    UnimplementedUpstreamClient,
    UnimplementedUpstreamClient,
    crate::agent::completions::mock::Client,
    StubFetcher,
    StubUsageHandler,
> {
    super::Client {
        mcp_client: Arc::new(crate::mcp::Client::new(
            reqwest::Client::new(),
            None,
            None,
            None,
            Duration::ZERO,
            Duration::ZERO,
            Duration::ZERO,
            0.0,
            1.0,
            Duration::ZERO,
            Duration::ZERO,
            Duration::from_millis(1),
        )),
        agent_fetcher: Arc::new(crate::agent::fetcher::CachingFetcher::new(
            Arc::new(StubFetcher),
        )),
        usage_handler: Arc::new(StubUsageHandler),
        openrouter: Arc::new(UnimplementedUpstreamClient),
        claude_agent_sdk: Arc::new(UnimplementedUpstreamClient),
        mock: Arc::new(crate::agent::completions::mock::Client {
            delay: Duration::ZERO,
        }),
        backoff_current_interval: Duration::ZERO,
        backoff_initial_interval: Duration::ZERO,
        backoff_randomization_factor: 0.0,
        backoff_multiplier: 1.0,
        backoff_max_interval: Duration::ZERO,
        backoff_max_elapsed_time: Duration::ZERO,
        first_chunk_timeout: Duration::from_millis(1),
        other_chunk_timeout: Duration::from_millis(1),
    }
}

fn make_ctx() -> ctx::Context<ctx::DefaultContextExt> {
    ctx::Context::new(
        Arc::new(ctx::DefaultContextExt),
        rust_decimal::Decimal::ONE,
    )
}

// ---------------------------------------------------------------------------
// Snapshot helpers
// ---------------------------------------------------------------------------

fn aggregate<S>(
    items: &[StreamItem<S>],
) -> AgentCompletion {
    let mut agg: Option<
        objectiveai::agent::completions::response::streaming::AgentCompletionChunk,
    > = None;
    for item in items {
        if let StreamItem::Chunk(chunk) = item {
            match &mut agg {
                Some(a) => a.push(chunk),
                None => agg = Some(chunk.clone()),
            }
        }
    }
    agg.expect("stream should have at least one chunk").into()
}

fn normalize(mut c: AgentCompletion) -> AgentCompletion {
    use objectiveai::agent::completions::response::unary::Message;
    c.id = String::new();
    c.created = 0;
    for msg in &mut c.messages {
        if let Message::Assistant(asst) = msg {
            asst.upstream_id = String::new();
            asst.created = 0;
        }
    }
    c
}

fn assert_snapshot(json: &str, path: &str, expected: &str) {
    if std::env::var("UPDATE_AGENT_COMPLETIONS_CLIENT_TESTS_SNAPSHOTS").as_deref() == Ok("1") {
        std::fs::write(path, json).unwrap();
        eprintln!("Updated snapshot: {path}");
        let written = std::fs::read_to_string(path).unwrap();
        assert_eq!(json, written.trim_end());
    } else {
        assert_eq!(json, expected.trim_end());
    }
}

fn assert_chunk_invariants<S>(items: &[StreamItem<S>]) {
    let chunks: Vec<_> = items
        .iter()
        .filter_map(|item| match item {
            StreamItem::Chunk(c) => Some(c),
            _ => None,
        })
        .collect();
    assert!(!chunks.is_empty(), "stream must have at least one chunk");
    for (i, chunk) in chunks.iter().enumerate() {
        assert!(
            chunk.messages.len() <= 1,
            "chunk {i} has {} messages, expected at most 1",
            chunk.messages.len(),
        );
        if i < chunks.len() - 1 {
            assert!(
                chunk.usage.is_none(),
                "chunk {i} (non-final) has usage, expected None",
            );
        } else {
            assert!(
                chunk.usage.is_some(),
                "final chunk {i} has no usage, expected Some",
            );
        }
    }
    assert!(
        matches!(items.last(), Some(StreamItem::State(_))),
        "final stream item must be a State",
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
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase::default(),
        )),
        agents: None,
        provider: None,
        response_format: None,
        seed: Some(42),
        stream: None,
        mcp_server_authorization: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, None)
        .await
        .expect("create_streaming should succeed");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert_chunk_invariants(&items);

    let completion = normalize(aggregate(&items));
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
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase::default(),
        )),
        agents: None,
        provider: None,
        response_format: None,
        seed: Some(123),
        stream: None,
        mcp_server_authorization: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, None)
        .await
        .expect("create_streaming should succeed");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert_chunk_invariants(&items);

    let completion = normalize(aggregate(&items));
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
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase::default(),
        )),
        agents: None,
        provider: None,
        response_format: None,
        seed: Some(77),
        stream: None,
        mcp_server_authorization: None,
    });

    let client_a = make_client();
    let stream_a = client_a
        .create_streaming(make_ctx(), params.clone(), None, None, None)
        .await
        .unwrap();
    let items_a: Vec<_> = Box::pin(stream_a).collect().await;
    assert_chunk_invariants(&items_a);

    let client_b = make_client();
    let stream_b = client_b
        .create_streaming(make_ctx(), params, None, None, None)
        .await
        .unwrap();
    let items_b: Vec<_> = Box::pin(stream_b).collect().await;
    assert_chunk_invariants(&items_b);

    let chunks_a: Vec<_> = items_a.iter().filter_map(|i| match i {
        StreamItem::Chunk(c) => Some(c),
        _ => None,
    }).collect();
    let chunks_b: Vec<_> = items_b.iter().filter_map(|i| match i {
        StreamItem::Chunk(c) => Some(c),
        _ => None,
    }).collect();
    assert_eq!(chunks_a.len(), chunks_b.len());

    let completion_a = normalize(aggregate(&items_a));
    let completion_b = normalize(aggregate(&items_b));
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
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase::default(),
        )),
        agents: None,
        provider: None,
        response_format: None,
        seed: Some(1),
        stream: None,
        mcp_server_authorization: None,
    });
    let params_b = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase::default(),
        )),
        agents: None,
        provider: None,
        response_format: None,
        seed: Some(2),
        stream: None,
        mcp_server_authorization: None,
    });

    let client_a = make_client();
    let stream_a = client_a
        .create_streaming(make_ctx(), params_a, None, None, None)
        .await
        .unwrap();
    let items_a: Vec<_> = Box::pin(stream_a).collect().await;
    assert_chunk_invariants(&items_a);

    let client_b = make_client();
    let stream_b = client_b
        .create_streaming(make_ctx(), params_b, None, None, None)
        .await
        .unwrap();
    let items_b: Vec<_> = Box::pin(stream_b).collect().await;
    assert_chunk_invariants(&items_b);

    let chunks_a: Vec<_> = items_a.iter().filter_map(|i| match i {
        StreamItem::Chunk(c) => Some(serde_json::to_string(c).unwrap()),
        _ => None,
    }).collect();
    let chunks_b: Vec<_> = items_b.iter().filter_map(|i| match i {
        StreamItem::Chunk(c) => Some(serde_json::to_string(c).unwrap()),
        _ => None,
    }).collect();
    assert_ne!(chunks_a, chunks_b);

    let completion_a = normalize(aggregate(&items_a));
    let completion_b = normalize(aggregate(&items_b));

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
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase {
                error: Some(true),
                ..Default::default()
            },
        )),
        agents: None,
        provider: None,
        response_format: None,
        seed: Some(42),
        stream: None,
        mcp_server_authorization: None,
    });

    let result = client
        .create_streaming(make_ctx(), params, None, None, None)
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
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase::default(),
        )),
        agents: None,
        provider: None,
        response_format: None,
        seed: Some(42),
        stream: None,
        mcp_server_authorization: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, None)
        .await
        .expect("should succeed with user message");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert_chunk_invariants(&items);

    let completion = normalize(aggregate(&items));
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
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase::default(),
        )),
        agents: None,
        provider: None,
        response_format: None,
        seed: Some(99),
        stream: None,
        mcp_server_authorization: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, None)
        .await
        .expect("should succeed with developer+user messages");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert_chunk_invariants(&items);

    let completion = normalize(aggregate(&items));
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
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase::default(),
        )),
        agents: None,
        provider: None,
        response_format: Some(ResponseFormatParam::Single(ResponseFormat::JsonObject)),
        seed: Some(42),
        stream: None,
        mcp_server_authorization: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, None)
        .await
        .expect("JsonObject should succeed");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert_chunk_invariants(&items);

    let completion = normalize(aggregate(&items));
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
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase::default(),
        )),
        agents: None,
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
        mcp_server_authorization: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, None)
        .await
        .expect("JsonSchema should succeed");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert_chunk_invariants(&items);

    let completion = normalize(aggregate(&items));
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
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase::default(),
        )),
        agents: None,
        provider: None,
        response_format: Some(ResponseFormatParam::Single(ResponseFormat::Text)),
        seed: Some(77),
        stream: None,
        mcp_server_authorization: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, None)
        .await
        .expect("Text should succeed");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert_chunk_invariants(&items);

    let completion = normalize(aggregate(&items));
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
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase::default(),
        )),
        agents: None,
        provider: None,
        response_format: Some(ResponseFormatParam::Single(ResponseFormat::Grammar {
            grammar: "root ::= 'hello'".into(),
        })),
        seed: Some(42),
        stream: None,
        mcp_server_authorization: None,
    });

    let result = client
        .create_streaming(make_ctx(), params, None, None, None)
        .await;
    assert!(result.is_err(), "Grammar should be rejected");
}

/// Response format: Python should be rejected by mock client.
#[tokio::test]
async fn test_python_response_format_rejected() {
    let client = make_client();
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase::default(),
        )),
        agents: None,
        provider: None,
        response_format: Some(ResponseFormatParam::Single(ResponseFormat::Python)),
        seed: Some(42),
        stream: None,
        mcp_server_authorization: None,
    });

    let result = client
        .create_streaming(make_ctx(), params, None, None, None)
        .await;
    assert!(result.is_err(), "Python should be rejected");
}

/// Response format: ToolCall with required=true.
#[tokio::test]
async fn test_required_tool_call_response_format() {
    let client = make_client();
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase::default(),
        )),
        agents: None,
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
        mcp_server_authorization: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, None)
        .await
        .expect("required ToolCall should succeed");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert_chunk_invariants(&items);

    let completion = normalize(aggregate(&items));
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
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase::default(),
        )),
        agents: None,
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
        mcp_server_authorization: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, None)
        .await
        .expect("optional ToolCall should succeed");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert_chunk_invariants(&items);

    let completion = normalize(aggregate(&items));
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/client_tests/test_optional_tool_call_response_format.json"),
        include_str!("../../../assets/agent/completions/client_tests/test_optional_tool_call_response_format.json"),
    );
}

/// With invention tools provided.
#[tokio::test]
async fn test_with_invention_tools() {
    let client = make_client();
    let inv1 = objectiveai::functions::inventions::InventionTool {
        name: "execute_code",
        description: "Execute code in a sandbox",
        parameters: indexmap::indexmap! {
            "type".into() => serde_json::json!("object"),
            "properties".into() => serde_json::json!({
                "language": {"type": "string"},
                "code": {"type": "string"},
            }),
        },
        call: Arc::new(|_| Box::pin(async { Ok("executed".into()) })),
    };
    let inv2 = objectiveai::functions::inventions::InventionTool {
        name: "read_file",
        description: "Read a file from disk",
        parameters: indexmap::indexmap! {
            "type".into() => serde_json::json!("object"),
            "properties".into() => serde_json::json!({
                "path": {"type": "string"},
            }),
        },
        call: Arc::new(|_| Box::pin(async { Ok("file contents".into()) })),
    };

    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![Message::User(UserMessage {
            content: RichContent::Text("Run some code".into()),
            name: None,
        })],
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase::default(),
        )),
        agents: None,
        provider: None,
        response_format: None,
        seed: Some(88),
        stream: None,
        mcp_server_authorization: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, Some(vec![inv1, inv2]), None)
        .await
        .expect("should succeed with invention tools");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert_chunk_invariants(&items);

    let completion = normalize(aggregate(&items));

    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/client_tests/test_with_invention_tools.json"),
        include_str!("../../../assets/agent/completions/client_tests/test_with_invention_tools.json"),
    );
}

/// With invention tools and ToolCall response format.
#[tokio::test]
async fn test_invention_tools_with_tool_call_response_format() {
    let client = make_client();
    let inv = objectiveai::functions::inventions::InventionTool {
        name: "validate",
        description: "Validate data",
        parameters: indexmap::indexmap! {
            "type".into() => serde_json::json!("object"),
            "properties".into() => serde_json::json!({
                "data": {"type": "string"},
            }),
        },
        call: Arc::new(|_| Box::pin(async { Ok("valid".into()) })),
    };

    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase::default(),
        )),
        agents: None,
        provider: None,
        response_format: Some(ResponseFormatParam::Single(ResponseFormat::ToolCall {
            name: "submit".into(),
            description: "Submit".into(),
            schema: indexmap::indexmap! {
                "type".into() => serde_json::json!("object"),
                "properties".into() => serde_json::json!({
                    "result": {"type": "string"},
                }),
            },
            required: None,
        })),
        seed: Some(150),
        stream: None,
        mcp_server_authorization: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, Some(vec![inv]), None)
        .await
        .expect("should succeed with invention tools and response format");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert_chunk_invariants(&items);

    let completion = normalize(aggregate(&items));

    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/client_tests/test_invention_tools_with_tool_call_response_format.json"),
        include_str!("../../../assets/agent/completions/client_tests/test_invention_tools_with_tool_call_response_format.json"),
    );
}

/// Single invention tool that returns an error.
#[tokio::test]
async fn test_invention_tool_returns_error() {
    let client = make_client();
    let inv = objectiveai::functions::inventions::InventionTool {
        name: "failing_tool",
        description: "A tool that always fails",
        parameters: indexmap::indexmap! {
            "type".into() => serde_json::json!("object"),
            "properties".into() => serde_json::json!({
                "input": {"type": "string"},
            }),
        },
        call: Arc::new(|_| Box::pin(async { Err("tool execution failed".into()) })),
    };

    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase::default(),
        )),
        agents: None,
        provider: None,
        response_format: None,
        seed: Some(88),
        stream: None,
        mcp_server_authorization: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, Some(vec![inv]), None)
        .await
        .expect("should succeed even with failing invention tool");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert_chunk_invariants(&items);

    let completion = normalize(aggregate(&items));
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/client_tests/test_invention_tool_returns_error.json"),
        include_str!("../../../assets/agent/completions/client_tests/test_invention_tool_returns_error.json"),
    );
}

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
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase::default(),
        )),
        agents: None,
        provider: None,
        response_format: None,
        seed: Some(55),
        stream: None,
        mcp_server_authorization: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, None)
        .await
        .expect("should succeed with multiple user messages");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert_chunk_invariants(&items);

    let completion = normalize(aggregate(&items));
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
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase {
                error: Some(false),
                ..Default::default()
            },
        )),
        agents: None,
        provider: None,
        response_format: None,
        seed: Some(42),
        stream: None,
        mcp_server_authorization: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, None)
        .await
        .expect("error=false should succeed");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert_chunk_invariants(&items);

    let completion = normalize(aggregate(&items));
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
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase::default(),
        )),
        agents: None,
        provider: None,
        response_format: None,
        seed: Some(42),
        stream: None,
        mcp_server_authorization: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, None)
        .await
        .unwrap();

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert_chunk_invariants(&items);
    match items.last() {
        Some(StreamItem::State(cont)) => {
            assert!(
                matches!(cont, crate::agent::completions::Continuation::Mock { .. }),
                "continuation should be Mock variant",
            );
        }
        other => panic!("expected State(Continuation::Mock), got {other:?}"),
    }

    let completion = normalize(aggregate(&items));
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
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(mock_base)),
        agents: None,
        provider: None,
        response_format: Some(ResponseFormatParam::PerAgent(per_agent)),
        seed: Some(42),
        stream: None,
        mcp_server_authorization: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, None)
        .await
        .expect("PerAgent response format should succeed");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert_chunk_invariants(&items);

    let completion = normalize(aggregate(&items));
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
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase::default(),
        )),
        agents: None,
        provider: None,
        response_format: Some(ResponseFormatParam::PerAgent(per_agent)),
        seed: Some(42),
        stream: None,
        mcp_server_authorization: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, None)
        .await
        .expect("PerAgent with unknown ID should succeed (no format applied)");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert_chunk_invariants(&items);

    let completion = normalize(aggregate(&items));
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
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase::default(),
        )),
        agents: None,
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
        mcp_server_authorization: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, None)
        .await
        .expect("nested JsonSchema should succeed");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert_chunk_invariants(&items);

    let completion = normalize(aggregate(&items));
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
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase {
                error: Some(true),
                ..Default::default()
            },
        )),
        agents: Some(vec![AgentParam::Provided(
            objectiveai::agent::AgentBase::Mock(MockAgentBase::default()),
        )]),
        provider: None,
        response_format: None,
        seed: Some(42),
        stream: None,
        mcp_server_authorization: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, None)
        .await
        .expect("fallback agent should succeed when primary errors");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert_chunk_invariants(&items);

    let completion = normalize(aggregate(&items));
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
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase {
                error: Some(true),
                ..Default::default()
            },
        )),
        agents: Some(vec![AgentParam::Provided(
            objectiveai::agent::AgentBase::Mock(MockAgentBase {
                error: Some(true),
                ..Default::default()
            }),
        )]),
        provider: None,
        response_format: None,
        seed: Some(42),
        stream: None,
        mcp_server_authorization: None,
    });

    let result = client
        .create_streaming(make_ctx(), params, None, None, None)
        .await;
    assert!(result.is_err(), "all agents erroring should fail");
}

/// Multiple fallback agents — first two error, third succeeds.
#[tokio::test]
async fn test_multiple_fallback_agents() {
    let client = make_client();
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase {
                error: Some(true),
                ..Default::default()
            },
        )),
        agents: Some(vec![
            AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
                MockAgentBase {
                    error: Some(true),
                    ..Default::default()
                },
            )),
            AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
                MockAgentBase::default(),
            )),
        ]),
        provider: None,
        response_format: None,
        seed: Some(42),
        stream: None,
        mcp_server_authorization: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, None)
        .await
        .expect("third agent should succeed");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert_chunk_invariants(&items);

    let completion = normalize(aggregate(&items));
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
    let mock_agent = objectiveai::agent::mock::Agent::try_from(MockAgentBase::default()).unwrap();

    let client = make_client();
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase::default(),
        )),
        agents: None,
        provider: None,
        response_format: None,
        seed: Some(42),
        stream: None,
        mcp_server_authorization: None,
    });

    let continuation = crate::agent::completions::Continuation::Mock {
        items: vec![
            crate::agent::completions::ContinuationItem::State(crate::agent::completions::mock::State::default()),
        ],
        agent: mock_agent,
        mcp_connections: vec![],
    };

    let stream = client
        .create_streaming(make_ctx(), params, Some(continuation), None, None)
        .await
        .expect("should succeed with continuation");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert_chunk_invariants(&items);

    let completion = normalize(aggregate(&items));
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
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase::default(),
        )),
        agents: None,
        provider: None,
        response_format: None,
        seed: Some(42),
        stream: None,
        mcp_server_authorization: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, None)
        .await
        .unwrap();

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert_chunk_invariants(&items);

    let state_count = items.iter().filter(|i| matches!(i, StreamItem::State(_))).count();
    assert_eq!(state_count, 1, "should have exactly one state");

    let completion = normalize(aggregate(&items));
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
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase::default(),
        )),
        agents: None,
        provider: None,
        response_format: None,
        seed: Some(u64::MAX as i64),
        stream: None,
        mcp_server_authorization: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, None)
        .await
        .expect("large seed should succeed");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert_chunk_invariants(&items);

    let completion = normalize(aggregate(&items));
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
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase::default(),
        )),
        agents: None,
        provider: None,
        response_format: None,
        seed: Some(0),
        stream: None,
        mcp_server_authorization: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, None)
        .await
        .expect("seed 0 should succeed");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert_chunk_invariants(&items);

    let completion = normalize(aggregate(&items));
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
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase {
                top_logprobs: Some(5),
                ..Default::default()
            },
        )),
        agents: None,
        provider: None,
        response_format: None,
        seed: Some(42),
        stream: None,
        mcp_server_authorization: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, None)
        .await
        .expect("logprobs basic should succeed");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert_chunk_invariants(&items);
    let completion = normalize(aggregate(&items));
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
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase {
                top_logprobs: Some(10),
                ..Default::default()
            },
        )),
        agents: None,
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
        mcp_server_authorization: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, None)
        .await
        .expect("logprobs json_schema nested should succeed");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert_chunk_invariants(&items);
    let completion = normalize(aggregate(&items));
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
#[tokio::test]
async fn test_logprobs_with_invention_tools() {
    let client = make_client();
    let inv = objectiveai::functions::inventions::InventionTool {
        name: "lookup",
        description: "Look up a value",
        parameters: indexmap::indexmap! {
            "type".into() => serde_json::json!("object"),
            "properties".into() => serde_json::json!({
                "key": {"type": "string"},
            }),
        },
        call: Arc::new(|_| Box::pin(async { Ok("found it".into()) })),
    };

    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![Message::User(UserMessage {
            content: RichContent::Text("Look up foo".into()),
            name: None,
        })],
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase {
                top_logprobs: Some(3),
                ..Default::default()
            },
        )),
        agents: None,
        provider: None,
        response_format: None,
        seed: Some(88),
        stream: None,
        mcp_server_authorization: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, Some(vec![inv]), None)
        .await
        .expect("logprobs with invention tools should succeed");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert_chunk_invariants(&items);
    let completion = normalize(aggregate(&items));

    // Assistant messages with content should have logprobs; those with
    // only tool_calls should not.
    for msg in &completion.messages {
        if let UnaryMessage::Assistant(asst) = msg {
            if asst.content.is_some() {
                assert!(asst.logprobs.is_some(), "content message missing logprobs");
            }
            if asst.tool_calls.is_some() && asst.content.is_none() {
                assert!(asst.logprobs.is_none(), "tool_call-only message has logprobs");
            }
        }
    }

    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/client_tests/test_logprobs_with_invention_tools.json"),
        include_str!("../../../assets/agent/completions/client_tests/test_logprobs_with_invention_tools.json"),
    );
}

/// Logprobs survive through the continuation flow.
#[tokio::test]
async fn test_logprobs_with_continuation() {
    let mock_base = MockAgentBase {
        top_logprobs: Some(7),
        ..Default::default()
    };
    let mock_agent = objectiveai::agent::mock::Agent::try_from(mock_base.clone()).unwrap();

    let client = make_client();
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(mock_base)),
        agents: None,
        provider: None,
        response_format: None,
        seed: Some(42),
        stream: None,
        mcp_server_authorization: None,
    });

    let continuation = crate::agent::completions::Continuation::Mock {
        items: vec![
            crate::agent::completions::ContinuationItem::State(
                crate::agent::completions::mock::State { tool_call_count: 3 },
            ),
        ],
        agent: mock_agent,
        mcp_connections: vec![],
    };

    let stream = client
        .create_streaming(make_ctx(), params, Some(continuation), None, None)
        .await
        .expect("logprobs with continuation should succeed");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert_chunk_invariants(&items);
    let completion = normalize(aggregate(&items));
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
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase {
                error: Some(true),
                ..Default::default()
            },
        )),
        agents: Some(vec![AgentParam::Provided(
            objectiveai::agent::AgentBase::Mock(MockAgentBase {
                top_logprobs: Some(12),
                ..Default::default()
            }),
        )]),
        provider: None,
        response_format: None,
        seed: Some(55),
        stream: None,
        mcp_server_authorization: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, None)
        .await
        .expect("fallback with logprobs should succeed");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert_chunk_invariants(&items);
    let completion = normalize(aggregate(&items));
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
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(mock_base)),
        agents: None,
        provider: None,
        response_format: Some(ResponseFormatParam::PerAgent(per_agent)),
        seed: Some(33),
        stream: None,
        mcp_server_authorization: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None, None)
        .await
        .expect("logprobs per-agent json_object should succeed");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert_chunk_invariants(&items);
    let completion = normalize(aggregate(&items));
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
