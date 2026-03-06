use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;

use objectiveai::agent::completions::request::{
    AgentCompletionCreateParams, Agent as AgentParam, ResponseFormat,
    ResponseFormatParam,
};
use objectiveai::agent::completions::response::streaming::AgentCompletionChunk;
use objectiveai::agent::completions::response::unary::{
    AgentCompletion, Message,
};
use objectiveai::agent::mock::{Agent, AgentBase};

use super::Client;
use crate::agent::completions::tool::{resolve_tools, ResolvedTool};
use crate::agent::completions::upstream_client::UpstreamClient;

fn default_agent() -> Agent {
    Agent::try_from(AgentBase::default()).unwrap()
}

fn default_params_with_seed(seed: i64) -> AgentCompletionCreateParams {
    AgentCompletionCreateParams {
        messages: vec![],
        agent: AgentParam::Id("mock".into()),
        provider: None,
        agents: None,
        response_format: None,
        seed: Some(seed),
        stream: None,
        mcp_server_authorization: None,
    }
}

fn params_with_response_format(seed: i64, rf: ResponseFormat) -> AgentCompletionCreateParams {
    AgentCompletionCreateParams {
        response_format: Some(ResponseFormatParam::Single(rf)),
        ..default_params_with_seed(seed)
    }
}

fn default_client() -> Client {
    Client {
        delay: Duration::ZERO,
    }
}

/// Runs the mock client to completion, accumulates all chunks, and returns AgentCompletion.
async fn run_mock(
    agent: &Agent,
    params: &AgentCompletionCreateParams,
    tool_names: &[String],
    tool_map: &HashMap<String, ResolvedTool>,
) -> AgentCompletion {
    let client = default_client();
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

// ---------------------------------------------------------------------------
// Snapshot helpers
// ---------------------------------------------------------------------------

fn normalize(mut c: AgentCompletion) -> AgentCompletion {
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
    if std::env::var("UPDATE_AGENT_COMPLETIONS_MOCK_CLIENT_TESTS_SNAPSHOTS").as_deref() == Ok("1") {
        std::fs::write(path, json).unwrap();
        eprintln!("Updated snapshot: {path}");
        let written = std::fs::read_to_string(path).unwrap();
        assert_eq!(json, written.trim_end());
    } else {
        assert_eq!(json, expected.trim_end());
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_no_tools_no_response_format_seed_42() {
    let completion = normalize(run_mock(
        &default_agent(),
        &default_params_with_seed(42),
        &[],
        &HashMap::new(),
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
    let completion = normalize(run_mock(
        &default_agent(),
        &default_params_with_seed(123),
        &[],
        &HashMap::new(),
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
    let completion = normalize(run_mock(
        &default_agent(),
        &default_params_with_seed(1),
        &[],
        &HashMap::new(),
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
    let completion = normalize(run_mock(
        &default_agent(),
        &default_params_with_seed(2),
        &[],
        &HashMap::new(),
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
    let agent = default_agent();
    let params = default_params_with_seed(123);
    let a = normalize(run_mock(&agent, &params, &[], &HashMap::new()).await);
    let b = normalize(run_mock(&agent, &params, &[], &HashMap::new()).await);
    assert_eq!(a, b);
}

#[tokio::test]
async fn test_different_seeds_differ() {
    let agent = default_agent();
    let a = normalize(run_mock(&agent, &default_params_with_seed(1), &[], &HashMap::new()).await);
    let b = normalize(run_mock(&agent, &default_params_with_seed(2), &[], &HashMap::new()).await);
    assert_ne!(a, b);
}

#[tokio::test]
async fn test_grammar_response_format_rejected() {
    let client = default_client();
    let agent = default_agent();
    let params = params_with_response_format(42, ResponseFormat::Grammar {
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
    let client = default_client();
    let agent = default_agent();
    let params = params_with_response_format(42, ResponseFormat::Python);

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
    let completion = normalize(run_mock(
        &default_agent(),
        &params_with_response_format(42, ResponseFormat::JsonObject),
        &[],
        &HashMap::new(),
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
        &[],
        &HashMap::new(),
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
    let completion = normalize(run_mock(
        &default_agent(),
        &params_with_response_format(77, ResponseFormat::Text),
        &[],
        &HashMap::new(),
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
    let completion = normalize(run_mock(
        &default_agent(),
        &default_params_with_seed(99),
        &tool_names,
        &tool_map,
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
    let (tool_names, tool_map) = resolve_tools(&[], &[], None, Some(&rf));

    let completion = normalize(run_mock(
        &default_agent(),
        &params,
        &tool_names,
        &tool_map,
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
    let completion = normalize(run_mock(
        &default_agent(),
        &default_params_with_seed(50),
        &tool_names,
        &tool_map,
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
    let completion = normalize(run_mock(
        &default_agent(),
        &default_params_with_seed(88),
        &tool_names,
        &tool_map,
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
    let completion = normalize(run_mock(
        &default_agent(),
        &default_params_with_seed(150),
        &tool_names,
        &tool_map,
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
    let params = params_with_response_format(200, rf.clone());
    let (tool_names, tool_map) = resolve_tools(&[conn], &[tools], Some(&[inv]), Some(&rf));
    let completion = normalize(run_mock(
        &default_agent(),
        &params,
        &tool_names,
        &tool_map,
    ).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/mock/client_tests/test_mcp_invention_and_response_format.json"),
        include_str!("../../../../assets/agent/completions/mock/client_tests/test_mcp_invention_and_response_format.json"),
    );
}
