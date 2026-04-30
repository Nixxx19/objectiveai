use std::sync::Arc;

use rust_decimal::Decimal;

use crate::ctx;

type Params = objectiveai::laboratories::executions::request::LaboratoryExecutionCreateParams;
type LaboratoryExecution = objectiveai::laboratories::executions::response::unary::LaboratoryExecution;
type LaboratoryExecutionChunk = objectiveai::laboratories::executions::response::streaming::LaboratoryExecutionChunk;
type TestClient = crate::test_clients::LaboratoryClient;

// ---------------------------------------------------------------------------
// Client constructor — delegates to the process-wide shared client.
// ---------------------------------------------------------------------------

fn make_client() -> Arc<TestClient> {
    crate::test_clients::laboratory()
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn builder_agent(seed_error: bool, error_probability: Option<u8>) -> objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional {
    objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(
        objectiveai::agent::InlineAgentBaseWithFallbacks {
            inner: objectiveai::agent::InlineAgentBase::Mock(objectiveai::agent::mock::AgentBase {
                mode: Some(objectiveai::agent::mock::Mode::LaboratoryBuilder),
                error: if seed_error { Some(true) } else { None },
                error_probability,
                ..Default::default()
            }),
            fallbacks: None,
        },
    )
}

fn evaluation_agent() -> objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional {
    objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(
        objectiveai::agent::InlineAgentBaseWithFallbacks {
            inner: objectiveai::agent::InlineAgentBase::Mock(objectiveai::agent::mock::AgentBase {
                mode: Some(objectiveai::agent::mock::Mode::LaboratoryEvaluation),
                ..Default::default()
            }),
            fallbacks: None,
        },
    )
}

fn user_message(text: &str) -> objectiveai::agent::completions::message::Message {
    objectiveai::agent::completions::message::Message::User(
        objectiveai::agent::completions::message::UserMessage {
            content: objectiveai::agent::completions::message::RichContent::Text(text.to_string()),
            name: None,
        },
    )
}

fn string_schema() -> objectiveai::functions::expression::InputSchema {
    objectiveai::functions::expression::InputSchema::String(
        objectiveai::functions::expression::StringInputSchema {
            r#type: objectiveai::functions::expression::StringInputSchemaType::String,
            description: None,
            r#enum: None,
        },
    )
}

fn make_request(
    builder_agents: Vec<objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional>,
    eval: bool,
    seed: i64,
) -> Arc<Params> {
    Arc::new(Params {
        docker_image: "alpine:3.23.3".to_string(),
        builder_agents,
        evaluation_agent: if eval { Some(evaluation_agent()) } else { None },
        builder_messages: vec![user_message("Build something.")],
        evaluation_messages: if eval { Some(vec![user_message("Evaluate the output.")]) } else { None },
        evaluation_output_schema: if eval { Some(string_schema()) } else { None },
        builder_continuation: None,
        evaluation_continuation: None,
        max_evaluation_retries: Some(1),
        persist: Some(false),
        provider: None,
        seed: Some(seed),
        stream: Some(true),
    })
}

fn make_ctx() -> ctx::Context<ctx::DefaultContextExt, ctx::persistent_cache::default::DefaultPersistentCacheClient> {
    ctx::Context::new(
        Arc::new(ctx::DefaultContextExt),
        Arc::new(ctx::persistent_cache::default::DefaultPersistentCacheClient),
        Decimal::ONE,
        true,
        &axum::http::HeaderMap::new(),
    )
}

async fn run_execution(client: &Arc<TestClient>, request: Arc<Params>) -> LaboratoryExecution {
    let ctx = make_ctx();
    let stream = client
        .clone()
        .create_streaming(ctx, request)
        .await
        .expect("create_streaming should succeed");
    let expected_created = std::cell::Cell::new(None);
    let agg = crate::stream_harness::consume_stream(
        Box::pin(stream),
        |agg, c| agg.push(c),
        |i, chunk| {
            check_created(&expected_created, i, chunk.created);
            assert!(chunk.usage.is_none(), "chunk {i} (non-final) has usage, expected None");
        },
        |i, chunk| {
            check_created(&expected_created, i, chunk.created);
            assert!(chunk.usage.is_none(), "chunk {i} (second-to-last) has usage, expected None");
        },
        |i, chunk| {
            check_created(&expected_created, i, chunk.created);
            assert!(chunk.usage.is_some(), "final chunk {i} has no usage, expected Some");
        },
    ).await;
    LaboratoryExecution::from(agg)
}

fn check_created(expected: &std::cell::Cell<Option<u64>>, _i: usize, created: u64) {
    match expected.get() {
        None => expected.set(Some(created)),
        Some(e) => assert_eq!(e, created, "created timestamp changed mid-stream"),
    }
}

fn normalize(mut exec: LaboratoryExecution) -> LaboratoryExecution {
    exec.normalize_for_tests();
    exec
}

fn assert_snapshot(json: &str, path: &str, expected: &str) {
    crate::stream_harness::assert_snapshot(
        json,
        path,
        expected,
        "UPDATE_LABORATORIES_EXECUTIONS_LOCAL_CLIENT_TESTS_SNAPSHOTS",
    );
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Single builder, no evaluation.
#[tokio::test]
async fn single_builder_no_eval_seed_42() {
    let client = make_client();
    let request = make_request(vec![builder_agent(false, None)], false, 42);
    let result = normalize(run_execution(&client, request).await);
    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/laboratories/executions/local/client_tests/single_builder_no_eval_seed_42.json"),
        include_str!("../../../assets/laboratories/executions/local/client_tests/single_builder_no_eval_seed_42.json"),
    );
}

/// Single builder + evaluation.
#[tokio::test]
async fn single_builder_with_eval_seed_42() {
    let client = make_client();
    let request = make_request(vec![builder_agent(false, None)], true, 42);
    let result = normalize(run_execution(&client, request).await);
    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/laboratories/executions/local/client_tests/single_builder_with_eval_seed_42.json"),
        include_str!("../../../assets/laboratories/executions/local/client_tests/single_builder_with_eval_seed_42.json"),
    );
}

/// Two builders + evaluation.
#[tokio::test]
async fn two_builders_with_eval_seed_99() {
    let client = make_client();
    let request = make_request(
        vec![builder_agent(false, None), builder_agent(false, None)],
        true,
        99,
    );
    let result = normalize(run_execution(&client, request).await);
    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/laboratories/executions/local/client_tests/two_builders_with_eval_seed_99.json"),
        include_str!("../../../assets/laboratories/executions/local/client_tests/two_builders_with_eval_seed_99.json"),
    );
}

/// Builder with 50% error probability + evaluation.
#[tokio::test]
async fn builder_error_50_with_eval_seed_10() {
    let client = make_client();
    let request = make_request(vec![builder_agent(true, Some(50))], true, 10);
    let result = normalize(run_execution(&client, request).await);
    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/laboratories/executions/local/client_tests/builder_error_50_with_eval_seed_10.json"),
        include_str!("../../../assets/laboratories/executions/local/client_tests/builder_error_50_with_eval_seed_10.json"),
    );
}

/// Two builders, one with 50% error probability, no evaluation.
#[tokio::test]
async fn two_builders_one_error_50_no_eval_seed_7() {
    let client = make_client();
    let request = make_request(
        vec![builder_agent(false, None), builder_agent(true, Some(50))],
        false,
        7,
    );
    let result = normalize(run_execution(&client, request).await);
    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/laboratories/executions/local/client_tests/two_builders_one_error_50_no_eval_seed_7.json"),
        include_str!("../../../assets/laboratories/executions/local/client_tests/two_builders_one_error_50_no_eval_seed_7.json"),
    );
}
