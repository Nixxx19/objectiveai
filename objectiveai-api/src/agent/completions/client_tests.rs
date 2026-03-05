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
use objectiveai::agent::completions::response::unary::AgentCompletion;
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

// ---------------------------------------------------------------------------
// Client constructor
// ---------------------------------------------------------------------------

fn make_client_with_tool_limit(
    seed: u64,
    max_tool_calls: Option<u32>,
) -> super::Client<
    ctx::DefaultContextExt,
    UnimplementedUpstreamClient,
    UnimplementedUpstreamClient,
    crate::agent::completions::mock::client::Client,
    StubFetcher,
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
        openrouter: Arc::new(UnimplementedUpstreamClient),
        claude_agent_sdk: Arc::new(UnimplementedUpstreamClient),
        mock: Arc::new(crate::agent::completions::mock::client::Client {
            delay: Duration::ZERO,
            seed: Some(seed),
            max_tool_calls,
            tool_call_count: Arc::new(std::sync::atomic::AtomicU32::new(0)),
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

fn make_client(
    seed: u64,
) -> super::Client<
    ctx::DefaultContextExt,
    UnimplementedUpstreamClient,
    UnimplementedUpstreamClient,
    crate::agent::completions::mock::client::Client,
    StubFetcher,
> {
    make_client_with_tool_limit(seed, None)
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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Default mock agent, no error.
#[tokio::test]
async fn test_basic_mock_agent_seed_42() {
    let client = make_client(42);
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase::default(),
        )),
        agents: None,
        provider: None,
        response_format: None,
        seed: None,
        stream: None,
        mcp_server_authorization: None,
        backoff_max_elapsed_time: None,
        first_chunk_timeout: None,
        other_chunk_timeout: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None)
        .await
        .expect("create_streaming should succeed");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert!(items.len() >= 2, "should have at least one chunk and one state");
    assert!(matches!(items.last(), Some(StreamItem::State(_))));

    let completion = normalize(aggregate(&items));
    let expected: AgentCompletion = serde_json::from_str(
        include_str!("../../../assets/agent/completions/client_tests/test_basic_mock_agent_seed_42.json")
    ).unwrap();
    assert_eq!(completion, expected);
}

/// Default mock agent with seed 123.
#[tokio::test]
async fn test_basic_mock_agent_seed_123() {
    let client = make_client(123);
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase::default(),
        )),
        agents: None,
        provider: None,
        response_format: None,
        seed: None,
        stream: None,
        mcp_server_authorization: None,
        backoff_max_elapsed_time: None,
        first_chunk_timeout: None,
        other_chunk_timeout: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None)
        .await
        .expect("create_streaming should succeed");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert!(items.len() >= 2);
    assert!(matches!(items.last(), Some(StreamItem::State(_))));

    let completion = normalize(aggregate(&items));
    let expected: AgentCompletion = serde_json::from_str(
        include_str!("../../../assets/agent/completions/client_tests/test_basic_mock_agent_seed_123.json")
    ).unwrap();
    assert_eq!(completion, expected);
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
        seed: None,
        stream: None,
        mcp_server_authorization: None,
        backoff_max_elapsed_time: None,
        first_chunk_timeout: None,
        other_chunk_timeout: None,
    });

    let client_a = make_client(77);
    let stream_a = client_a
        .create_streaming(make_ctx(), params.clone(), None, None)
        .await
        .unwrap();
    let items_a: Vec<_> = Box::pin(stream_a).collect().await;

    let client_b = make_client(77);
    let stream_b = client_b
        .create_streaming(make_ctx(), params, None, None)
        .await
        .unwrap();
    let items_b: Vec<_> = Box::pin(stream_b).collect().await;

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

    let expected: AgentCompletion = serde_json::from_str(
        include_str!("../../../assets/agent/completions/client_tests/test_deterministic_with_same_seed.json")
    ).unwrap();
    assert_eq!(completion_a, expected);
}

/// Different seeds produce different streams.
#[tokio::test]
async fn test_different_seeds_differ() {
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase::default(),
        )),
        agents: None,
        provider: None,
        response_format: None,
        seed: None,
        stream: None,
        mcp_server_authorization: None,
        backoff_max_elapsed_time: None,
        first_chunk_timeout: None,
        other_chunk_timeout: None,
    });

    let client_a = make_client(1);
    let stream_a = client_a
        .create_streaming(make_ctx(), params.clone(), None, None)
        .await
        .unwrap();
    let items_a: Vec<_> = Box::pin(stream_a).collect().await;

    let client_b = make_client(2);
    let stream_b = client_b
        .create_streaming(make_ctx(), params, None, None)
        .await
        .unwrap();
    let items_b: Vec<_> = Box::pin(stream_b).collect().await;

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

    let expected_a: AgentCompletion = serde_json::from_str(
        include_str!("../../../assets/agent/completions/client_tests/test_different_seeds_differ_a.json")
    ).unwrap();
    assert_eq!(completion_a, expected_a);

    let expected_b: AgentCompletion = serde_json::from_str(
        include_str!("../../../assets/agent/completions/client_tests/test_different_seeds_differ_b.json")
    ).unwrap();
    assert_eq!(completion_b, expected_b);
}

/// Mock agent with error=true should fail.
#[tokio::test]
async fn test_mock_agent_with_error() {
    let client = make_client(42);
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
        seed: None,
        stream: None,
        mcp_server_authorization: None,
        backoff_max_elapsed_time: None,
        first_chunk_timeout: None,
        other_chunk_timeout: None,
    });

    let result = client
        .create_streaming(make_ctx(), params, None, None)
        .await;
    assert!(result.is_err(), "error agent should fail");
}

/// Messages: single user message.
#[tokio::test]
async fn test_with_single_user_message() {
    let client = make_client(42);
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
        seed: None,
        stream: None,
        mcp_server_authorization: None,
        backoff_max_elapsed_time: None,
        first_chunk_timeout: None,
        other_chunk_timeout: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None)
        .await
        .expect("should succeed with user message");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert!(items.len() >= 2);
    assert!(matches!(items.last(), Some(StreamItem::State(_))));

    let completion = normalize(aggregate(&items));
    let expected: AgentCompletion = serde_json::from_str(
        include_str!("../../../assets/agent/completions/client_tests/test_with_single_user_message.json")
    ).unwrap();
    assert_eq!(completion, expected);
}

/// Messages: developer + user messages.
#[tokio::test]
async fn test_with_developer_and_user_messages() {
    let client = make_client(99);
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
        seed: None,
        stream: None,
        mcp_server_authorization: None,
        backoff_max_elapsed_time: None,
        first_chunk_timeout: None,
        other_chunk_timeout: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None)
        .await
        .expect("should succeed with developer+user messages");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert!(items.len() >= 2);
    assert!(matches!(items.last(), Some(StreamItem::State(_))));

    let completion = normalize(aggregate(&items));
    let expected: AgentCompletion = serde_json::from_str(
        include_str!("../../../assets/agent/completions/client_tests/test_with_developer_and_user_messages.json")
    ).unwrap();
    assert_eq!(completion, expected);
}

/// Response format: JsonObject.
#[tokio::test]
async fn test_json_object_response_format() {
    let client = make_client(42);
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase::default(),
        )),
        agents: None,
        provider: None,
        response_format: Some(ResponseFormatParam::Single(ResponseFormat::JsonObject)),
        seed: None,
        stream: None,
        mcp_server_authorization: None,
        backoff_max_elapsed_time: None,
        first_chunk_timeout: None,
        other_chunk_timeout: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None)
        .await
        .expect("JsonObject should succeed");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert!(items.len() >= 2);
    assert!(matches!(items.last(), Some(StreamItem::State(_))));

    let completion = normalize(aggregate(&items));
    let expected: AgentCompletion = serde_json::from_str(
        include_str!("../../../assets/agent/completions/client_tests/test_json_object_response_format.json")
    ).unwrap();
    assert_eq!(completion, expected);
}

/// Response format: JsonSchema with object schema.
#[tokio::test]
async fn test_json_schema_response_format() {
    let client = make_client(42);
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
        seed: None,
        stream: None,
        mcp_server_authorization: None,
        backoff_max_elapsed_time: None,
        first_chunk_timeout: None,
        other_chunk_timeout: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None)
        .await
        .expect("JsonSchema should succeed");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert!(items.len() >= 2);
    assert!(matches!(items.last(), Some(StreamItem::State(_))));

    let completion = normalize(aggregate(&items));
    let expected: AgentCompletion = serde_json::from_str(
        include_str!("../../../assets/agent/completions/client_tests/test_json_schema_response_format.json")
    ).unwrap();
    assert_eq!(completion, expected);
}

/// Response format: Text.
#[tokio::test]
async fn test_text_response_format() {
    let client = make_client(77);
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase::default(),
        )),
        agents: None,
        provider: None,
        response_format: Some(ResponseFormatParam::Single(ResponseFormat::Text)),
        seed: None,
        stream: None,
        mcp_server_authorization: None,
        backoff_max_elapsed_time: None,
        first_chunk_timeout: None,
        other_chunk_timeout: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None)
        .await
        .expect("Text should succeed");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert!(items.len() >= 2);
    assert!(matches!(items.last(), Some(StreamItem::State(_))));

    let completion = normalize(aggregate(&items));
    let expected: AgentCompletion = serde_json::from_str(
        include_str!("../../../assets/agent/completions/client_tests/test_text_response_format.json")
    ).unwrap();
    assert_eq!(completion, expected);
}

/// Response format: Grammar should be rejected by mock client.
#[tokio::test]
async fn test_grammar_response_format_rejected() {
    let client = make_client(42);
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
        seed: None,
        stream: None,
        mcp_server_authorization: None,
        backoff_max_elapsed_time: None,
        first_chunk_timeout: None,
        other_chunk_timeout: None,
    });

    let result = client
        .create_streaming(make_ctx(), params, None, None)
        .await;
    assert!(result.is_err(), "Grammar should be rejected");
}

/// Response format: Python should be rejected by mock client.
#[tokio::test]
async fn test_python_response_format_rejected() {
    let client = make_client(42);
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase::default(),
        )),
        agents: None,
        provider: None,
        response_format: Some(ResponseFormatParam::Single(ResponseFormat::Python)),
        seed: None,
        stream: None,
        mcp_server_authorization: None,
        backoff_max_elapsed_time: None,
        first_chunk_timeout: None,
        other_chunk_timeout: None,
    });

    let result = client
        .create_streaming(make_ctx(), params, None, None)
        .await;
    assert!(result.is_err(), "Python should be rejected");
}

/// Response format: ToolCall with required=true.
#[tokio::test]
async fn test_required_tool_call_response_format() {
    let client = make_client(42);
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
        seed: None,
        stream: None,
        mcp_server_authorization: None,
        backoff_max_elapsed_time: None,
        first_chunk_timeout: None,
        other_chunk_timeout: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None)
        .await
        .expect("required ToolCall should succeed");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert!(items.len() >= 2);
    assert!(matches!(items.last(), Some(StreamItem::State(_))));

    let completion = normalize(aggregate(&items));
    let expected: AgentCompletion = serde_json::from_str(
        include_str!("../../../assets/agent/completions/client_tests/test_required_tool_call_response_format.json")
    ).unwrap();
    assert_eq!(completion, expected);
}

/// Response format: ToolCall with required=None (optional).
#[tokio::test]
async fn test_optional_tool_call_response_format() {
    let client = make_client(200);
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
        seed: None,
        stream: None,
        mcp_server_authorization: None,
        backoff_max_elapsed_time: None,
        first_chunk_timeout: None,
        other_chunk_timeout: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None)
        .await
        .expect("optional ToolCall should succeed");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert!(items.len() >= 2);
    assert!(matches!(items.last(), Some(StreamItem::State(_))));

    let completion = normalize(aggregate(&items));
    let expected: AgentCompletion = serde_json::from_str(
        include_str!("../../../assets/agent/completions/client_tests/test_optional_tool_call_response_format.json")
    ).unwrap();
    assert_eq!(completion, expected);
}

/// With invention tools provided.
#[tokio::test]
async fn test_with_invention_tools() {
    let client = make_client_with_tool_limit(88, Some(3));
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
        seed: None,
        stream: None,
        mcp_server_authorization: None,
        backoff_max_elapsed_time: None,
        first_chunk_timeout: None,
        other_chunk_timeout: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, Some(vec![inv1, inv2]))
        .await
        .expect("should succeed with invention tools");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert!(items.len() >= 2);
    assert!(matches!(items.last(), Some(StreamItem::State(_))));

    let completion = normalize(aggregate(&items));
    let expected: AgentCompletion = serde_json::from_str(
        include_str!("../../../assets/agent/completions/client_tests/test_with_invention_tools.json")
    ).unwrap();
    assert_eq!(completion, expected);
}

/// With invention tools and ToolCall response format.
#[tokio::test]
async fn test_invention_tools_with_tool_call_response_format() {
    let client = make_client_with_tool_limit(150, Some(3));
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
        seed: None,
        stream: None,
        mcp_server_authorization: None,
        backoff_max_elapsed_time: None,
        first_chunk_timeout: None,
        other_chunk_timeout: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, Some(vec![inv]))
        .await
        .expect("should succeed with invention tools and response format");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert!(items.len() >= 2);
    assert!(matches!(items.last(), Some(StreamItem::State(_))));

    let completion = normalize(aggregate(&items));
    let expected: AgentCompletion = serde_json::from_str(
        include_str!("../../../assets/agent/completions/client_tests/test_invention_tools_with_tool_call_response_format.json")
    ).unwrap();
    assert_eq!(completion, expected);
}

/// Single invention tool that returns an error.
#[tokio::test]
async fn test_invention_tool_returns_error() {
    let client = make_client_with_tool_limit(88, Some(3));
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
        seed: None,
        stream: None,
        mcp_server_authorization: None,
        backoff_max_elapsed_time: None,
        first_chunk_timeout: None,
        other_chunk_timeout: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, Some(vec![inv]))
        .await
        .expect("should succeed even with failing invention tool");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert!(items.len() >= 2);
    assert!(matches!(items.last(), Some(StreamItem::State(_))));

    let completion = normalize(aggregate(&items));
    let expected: AgentCompletion = serde_json::from_str(
        include_str!("../../../assets/agent/completions/client_tests/test_invention_tool_returns_error.json")
    ).unwrap();
    assert_eq!(completion, expected);
}

/// Multiple user messages in a conversation.
#[tokio::test]
async fn test_multiple_user_messages() {
    let client = make_client(55);
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
        seed: None,
        stream: None,
        mcp_server_authorization: None,
        backoff_max_elapsed_time: None,
        first_chunk_timeout: None,
        other_chunk_timeout: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None)
        .await
        .expect("should succeed with multiple user messages");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert!(items.len() >= 2);
    assert!(matches!(items.last(), Some(StreamItem::State(_))));

    let completion = normalize(aggregate(&items));
    let expected: AgentCompletion = serde_json::from_str(
        include_str!("../../../assets/agent/completions/client_tests/test_multiple_user_messages.json")
    ).unwrap();
    assert_eq!(completion, expected);
}

/// Mock agent with error=Some(false) should succeed (normalized to None by prepare).
#[tokio::test]
async fn test_mock_agent_error_false_succeeds() {
    let client = make_client(42);
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
        seed: None,
        stream: None,
        mcp_server_authorization: None,
        backoff_max_elapsed_time: None,
        first_chunk_timeout: None,
        other_chunk_timeout: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None)
        .await
        .expect("error=false should succeed");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert!(items.len() >= 2);
    assert!(matches!(items.last(), Some(StreamItem::State(_))));

    let completion = normalize(aggregate(&items));
    let expected: AgentCompletion = serde_json::from_str(
        include_str!("../../../assets/agent/completions/client_tests/test_mock_agent_error_false_succeeds.json")
    ).unwrap();
    assert_eq!(completion, expected);
}

/// Final stream item is always a Continuation::Mock.
#[tokio::test]
async fn test_final_item_is_mock_continuation() {
    let client = make_client(42);
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase::default(),
        )),
        agents: None,
        provider: None,
        response_format: None,
        seed: None,
        stream: None,
        mcp_server_authorization: None,
        backoff_max_elapsed_time: None,
        first_chunk_timeout: None,
        other_chunk_timeout: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None)
        .await
        .unwrap();

    let items: Vec<_> = Box::pin(stream).collect().await;
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
    let expected: AgentCompletion = serde_json::from_str(
        include_str!("../../../assets/agent/completions/client_tests/test_final_item_is_mock_continuation.json")
    ).unwrap();
    assert_eq!(completion, expected);
}

/// PerAgent response format targeting the mock agent's ID.
#[tokio::test]
async fn test_per_agent_response_format() {
    let mock_base = MockAgentBase::default();
    let agent_id = mock_base.id();

    let client = make_client(42);
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
        seed: None,
        stream: None,
        mcp_server_authorization: None,
        backoff_max_elapsed_time: None,
        first_chunk_timeout: None,
        other_chunk_timeout: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None)
        .await
        .expect("PerAgent response format should succeed");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert!(items.len() >= 2);
    assert!(matches!(items.last(), Some(StreamItem::State(_))));

    let completion = normalize(aggregate(&items));
    let expected: AgentCompletion = serde_json::from_str(
        include_str!("../../../assets/agent/completions/client_tests/test_per_agent_response_format.json")
    ).unwrap();
    assert_eq!(completion, expected);
}

/// PerAgent response format with unknown agent ID (should fall back to no format).
#[tokio::test]
async fn test_per_agent_response_format_unknown_id() {
    let client = make_client(42);
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
        seed: None,
        stream: None,
        mcp_server_authorization: None,
        backoff_max_elapsed_time: None,
        first_chunk_timeout: None,
        other_chunk_timeout: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None)
        .await
        .expect("PerAgent with unknown ID should succeed (no format applied)");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert!(items.len() >= 2);
    assert!(matches!(items.last(), Some(StreamItem::State(_))));

    let completion = normalize(aggregate(&items));
    let expected: AgentCompletion = serde_json::from_str(
        include_str!("../../../assets/agent/completions/client_tests/test_per_agent_response_format_unknown_id.json")
    ).unwrap();
    assert_eq!(completion, expected);
}

/// JsonSchema with nested object schema.
#[tokio::test]
async fn test_json_schema_nested_object() {
    let client = make_client(99);
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
        seed: None,
        stream: None,
        mcp_server_authorization: None,
        backoff_max_elapsed_time: None,
        first_chunk_timeout: None,
        other_chunk_timeout: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None)
        .await
        .expect("nested JsonSchema should succeed");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert!(items.len() >= 2);
    assert!(matches!(items.last(), Some(StreamItem::State(_))));

    let completion = normalize(aggregate(&items));
    let expected: AgentCompletion = serde_json::from_str(
        include_str!("../../../assets/agent/completions/client_tests/test_json_schema_nested_object.json")
    ).unwrap();
    assert_eq!(completion, expected);
}

/// Fallback agents: primary errors, fallback succeeds.
#[tokio::test]
async fn test_fallback_agent_on_error() {
    let client = make_client(42);
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
        seed: None,
        stream: None,
        mcp_server_authorization: None,
        backoff_max_elapsed_time: None,
        first_chunk_timeout: None,
        other_chunk_timeout: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None)
        .await
        .expect("fallback agent should succeed when primary errors");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert!(items.len() >= 2);
    assert!(matches!(items.last(), Some(StreamItem::State(_))));

    let completion = normalize(aggregate(&items));
    let expected: AgentCompletion = serde_json::from_str(
        include_str!("../../../assets/agent/completions/client_tests/test_fallback_agent_on_error.json")
    ).unwrap();
    assert_eq!(completion, expected);
}

/// Both primary and fallback agents error — should fail.
#[tokio::test]
async fn test_all_agents_error() {
    let client = make_client(42);
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
        seed: None,
        stream: None,
        mcp_server_authorization: None,
        backoff_max_elapsed_time: None,
        first_chunk_timeout: None,
        other_chunk_timeout: None,
    });

    let result = client
        .create_streaming(make_ctx(), params, None, None)
        .await;
    assert!(result.is_err(), "all agents erroring should fail");
}

/// Multiple fallback agents — first two error, third succeeds.
#[tokio::test]
async fn test_multiple_fallback_agents() {
    let client = make_client(42);
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
        seed: None,
        stream: None,
        mcp_server_authorization: None,
        backoff_max_elapsed_time: None,
        first_chunk_timeout: None,
        other_chunk_timeout: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None)
        .await
        .expect("third agent should succeed");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert!(items.len() >= 2);
    assert!(matches!(items.last(), Some(StreamItem::State(_))));

    let completion = normalize(aggregate(&items));
    let expected: AgentCompletion = serde_json::from_str(
        include_str!("../../../assets/agent/completions/client_tests/test_multiple_fallback_agents.json")
    ).unwrap();
    assert_eq!(completion, expected);
}

/// With continuation from a previous Mock run.
#[tokio::test]
async fn test_with_mock_continuation() {
    let mock_agent = objectiveai::agent::mock::Agent::try_from(MockAgentBase::default()).unwrap();

    let client = make_client(42);
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase::default(),
        )),
        agents: None,
        provider: None,
        response_format: None,
        seed: None,
        stream: None,
        mcp_server_authorization: None,
        backoff_max_elapsed_time: None,
        first_chunk_timeout: None,
        other_chunk_timeout: None,
    });

    let continuation = crate::agent::completions::Continuation::Mock {
        items: vec![
            crate::agent::completions::ContinuationItem::State(()),
        ],
        agent: mock_agent,
        mcp_connections: vec![],
    };

    let stream = client
        .create_streaming(make_ctx(), params, Some(continuation), None)
        .await
        .expect("should succeed with continuation");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert!(items.len() >= 2);
    assert!(matches!(items.last(), Some(StreamItem::State(_))));

    let completion = normalize(aggregate(&items));
    let expected: AgentCompletion = serde_json::from_str(
        include_str!("../../../assets/agent/completions/client_tests/test_with_mock_continuation.json")
    ).unwrap();
    assert_eq!(completion, expected);
}

/// Stream produces chunks before the final state.
#[tokio::test]
async fn test_stream_yields_chunks_before_state() {
    let client = make_client(42);
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase::default(),
        )),
        agents: None,
        provider: None,
        response_format: None,
        seed: None,
        stream: None,
        mcp_server_authorization: None,
        backoff_max_elapsed_time: None,
        first_chunk_timeout: None,
        other_chunk_timeout: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None)
        .await
        .unwrap();

    let items: Vec<_> = Box::pin(stream).collect().await;

    let chunk_count = items.iter().filter(|i| matches!(i, StreamItem::Chunk(_))).count();
    let state_count = items.iter().filter(|i| matches!(i, StreamItem::State(_))).count();

    assert!(chunk_count >= 1, "should have at least one chunk");
    assert_eq!(state_count, 1, "should have exactly one state");
    assert!(
        matches!(items.last(), Some(StreamItem::State(_))),
        "state should be the last item",
    );

    let completion = normalize(aggregate(&items));
    let expected: AgentCompletion = serde_json::from_str(
        include_str!("../../../assets/agent/completions/client_tests/test_stream_yields_chunks_before_state.json")
    ).unwrap();
    assert_eq!(completion, expected);
}

/// Large seed value.
#[tokio::test]
async fn test_large_seed_value() {
    let client = make_client(u64::MAX);
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase::default(),
        )),
        agents: None,
        provider: None,
        response_format: None,
        seed: None,
        stream: None,
        mcp_server_authorization: None,
        backoff_max_elapsed_time: None,
        first_chunk_timeout: None,
        other_chunk_timeout: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None)
        .await
        .expect("large seed should succeed");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert!(items.len() >= 2);
    assert!(matches!(items.last(), Some(StreamItem::State(_))));

    let completion = normalize(aggregate(&items));
    let expected: AgentCompletion = serde_json::from_str(
        include_str!("../../../assets/agent/completions/client_tests/test_large_seed_value.json")
    ).unwrap();
    assert_eq!(completion, expected);
}

/// Seed 0.
#[tokio::test]
async fn test_seed_zero() {
    let client = make_client(0);
    let params = Arc::new(AgentCompletionCreateParams {
        messages: vec![],
        agent: AgentParam::Provided(objectiveai::agent::AgentBase::Mock(
            MockAgentBase::default(),
        )),
        agents: None,
        provider: None,
        response_format: None,
        seed: None,
        stream: None,
        mcp_server_authorization: None,
        backoff_max_elapsed_time: None,
        first_chunk_timeout: None,
        other_chunk_timeout: None,
    });

    let stream = client
        .create_streaming(make_ctx(), params, None, None)
        .await
        .expect("seed 0 should succeed");

    let items: Vec<_> = Box::pin(stream).collect().await;
    assert!(items.len() >= 2);
    assert!(matches!(items.last(), Some(StreamItem::State(_))));

    let completion = normalize(aggregate(&items));
    let expected: AgentCompletion = serde_json::from_str(
        include_str!("../../../assets/agent/completions/client_tests/test_seed_zero.json")
    ).unwrap();
    assert_eq!(completion, expected);
}
