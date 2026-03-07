//! Tests for function execution client.

use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use rust_decimal::Decimal;

use objectiveai::functions::executions::request::{
    FunctionRemoteProfileRemoteRequestBody, FunctionRemoteProfileRemoteRequestPath, Request,
    Strategy,
};
use objectiveai::functions::executions::response::streaming::FunctionExecutionChunk;
use objectiveai::functions::executions::response::unary::FunctionExecution;
use objectiveai::functions::expression::Input;
use objectiveai::functions::Remote;
use objectiveai::error::StatusError;

use crate::agent::completions::UnimplementedUpstreamClient;
use crate::ctx;
use crate::functions;

// ---------------------------------------------------------------------------
// Stubs
// ---------------------------------------------------------------------------

struct StubAgentFetcher;

#[async_trait::async_trait]
impl crate::agent::fetcher::Fetcher<ctx::DefaultContextExt> for StubAgentFetcher {
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
            message: serde_json::json!("stub agent fetcher should not be called"),
        })
    }
}

struct StubEnsembleFetcher;

#[async_trait::async_trait]
impl crate::ensemble::fetcher::Fetcher<ctx::DefaultContextExt> for StubEnsembleFetcher {
    async fn fetch(
        &self,
        _ctx: ctx::Context<ctx::DefaultContextExt>,
        _id: &str,
    ) -> Result<
        Option<(objectiveai::ensemble::Ensemble, u64)>,
        objectiveai::error::ResponseError,
    > {
        Err(objectiveai::error::ResponseError {
            code: 501,
            message: serde_json::json!("stub ensemble fetcher should not be called"),
        })
    }
}

struct StubCompletionVotesFetcher;

#[async_trait::async_trait]
impl crate::vector::completions::completion_votes_fetcher::Fetcher<ctx::DefaultContextExt>
    for StubCompletionVotesFetcher
{
    async fn fetch(
        &self,
        _ctx: ctx::Context<ctx::DefaultContextExt>,
        _id: &str,
    ) -> Result<
        Option<Vec<objectiveai::vector::completions::response::Vote>>,
        objectiveai::error::ResponseError,
    > {
        Ok(None)
    }
}

struct StubCacheVoteFetcher;

#[async_trait::async_trait]
impl crate::vector::completions::cache_vote_fetcher::Fetcher<ctx::DefaultContextExt>
    for StubCacheVoteFetcher
{
    async fn fetch(
        &self,
        _ctx: ctx::Context<ctx::DefaultContextExt>,
        _agent: &objectiveai::agent::completions::request::Agent,
        _agents: Option<&[objectiveai::agent::completions::request::Agent]>,
        _messages: &[objectiveai::agent::completions::message::Message],
        _responses: &[objectiveai::agent::completions::message::RichContent],
    ) -> Result<
        Option<objectiveai::vector::completions::response::Vote>,
        objectiveai::error::ResponseError,
    > {
        Ok(None)
    }
}

struct StubAgentUsageHandler;

impl crate::agent::completions::usage_handler::UsageHandler<ctx::DefaultContextExt>
    for StubAgentUsageHandler
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

struct StubVectorUsageHandler;

#[async_trait::async_trait]
impl crate::vector::completions::usage_handler::UsageHandler<ctx::DefaultContextExt>
    for StubVectorUsageHandler
{
    async fn handle_usage(
        &self,
        _ctx: ctx::Context<ctx::DefaultContextExt>,
        _request: Arc<objectiveai::vector::completions::request::VectorCompletionCreateParams>,
        _response: objectiveai::vector::completions::response::unary::VectorCompletion,
    ) {
    }
}

struct StubFunctionUsageHandler;

#[async_trait::async_trait]
impl super::usage_handler::UsageHandler<ctx::DefaultContextExt> for StubFunctionUsageHandler {
    async fn handle_usage(
        &self,
        _ctx: ctx::Context<ctx::DefaultContextExt>,
        _request: Arc<Request>,
        _response: FunctionExecution,
    ) {
    }
}

struct StubFunctionGithubFetcher;

#[async_trait::async_trait]
impl functions::function_fetcher::Fetcher<ctx::DefaultContextExt> for StubFunctionGithubFetcher {
    async fn fetch(
        &self,
        _ctx: ctx::Context<ctx::DefaultContextExt>,
        _owner: &str,
        _repository: &str,
        _commit: Option<&str>,
    ) -> Result<
        Option<functions::function_fetcher::FullGetFunction>,
        objectiveai::error::ResponseError,
    > {
        Err(objectiveai::error::ResponseError {
            code: 501,
            message: serde_json::json!("stub github function fetcher should not be called"),
        })
    }
}

struct StubProfileGithubFetcher;

#[async_trait::async_trait]
impl functions::profile_fetcher::Fetcher<ctx::DefaultContextExt> for StubProfileGithubFetcher {
    async fn fetch(
        &self,
        _ctx: ctx::Context<ctx::DefaultContextExt>,
        _owner: &str,
        _repository: &str,
        _commit: Option<&str>,
    ) -> Result<
        Option<objectiveai::functions::profiles::response::GetProfile>,
        objectiveai::error::ResponseError,
    > {
        Err(objectiveai::error::ResponseError {
            code: 501,
            message: serde_json::json!("stub github profile fetcher should not be called"),
        })
    }
}

struct StubFilesystemFetcher;

#[async_trait::async_trait]
impl functions::function_fetcher::Fetcher<ctx::DefaultContextExt> for StubFilesystemFetcher {
    async fn fetch(
        &self,
        _ctx: ctx::Context<ctx::DefaultContextExt>,
        _owner: &str,
        _repository: &str,
        _commit: Option<&str>,
    ) -> Result<
        Option<functions::function_fetcher::FullGetFunction>,
        objectiveai::error::ResponseError,
    > {
        Err(objectiveai::error::ResponseError {
            code: 501,
            message: serde_json::json!("stub filesystem function fetcher should not be called"),
        })
    }
}

struct StubFilesystemProfileFetcher;

#[async_trait::async_trait]
impl functions::profile_fetcher::Fetcher<ctx::DefaultContextExt>
    for StubFilesystemProfileFetcher
{
    async fn fetch(
        &self,
        _ctx: ctx::Context<ctx::DefaultContextExt>,
        _owner: &str,
        _repository: &str,
        _commit: Option<&str>,
    ) -> Result<
        Option<objectiveai::functions::profiles::response::GetProfile>,
        objectiveai::error::ResponseError,
    > {
        Err(objectiveai::error::ResponseError {
            code: 501,
            message: serde_json::json!("stub filesystem profile fetcher should not be called"),
        })
    }
}

// ---------------------------------------------------------------------------
// Client construction
// ---------------------------------------------------------------------------

type TestClient = super::Client<
    ctx::DefaultContextExt,
    UnimplementedUpstreamClient,
    UnimplementedUpstreamClient,
    crate::agent::completions::mock::Client,
    StubAgentFetcher,
    StubAgentUsageHandler,
    StubEnsembleFetcher,
    StubCompletionVotesFetcher,
    StubCacheVoteFetcher,
    StubVectorUsageHandler,
    StubFunctionGithubFetcher,
    StubFilesystemFetcher,
    functions::function_fetcher::mock::MockFetcher,
    StubProfileGithubFetcher,
    StubFilesystemProfileFetcher,
    functions::profile_fetcher::mock::MockFetcher,
    StubFunctionUsageHandler,
>;

fn make_client() -> Arc<TestClient> {
    let agent_client = Arc::new(crate::agent::completions::Client {
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
            Arc::new(StubAgentFetcher),
        )),
        usage_handler: Arc::new(StubAgentUsageHandler),
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
    });
    let ensemble_fetcher = Arc::new(crate::ensemble::fetcher::CachingFetcher::new(
        Arc::new(StubEnsembleFetcher),
    ));
    let vector_client = Arc::new(crate::vector::completions::Client {
        agent_client: agent_client.clone(),
        ensemble_fetcher: ensemble_fetcher.clone(),
        completion_votes_fetcher: Arc::new(StubCompletionVotesFetcher),
        cache_vote_fetcher: Arc::new(StubCacheVoteFetcher),
        usage_handler: Arc::new(StubVectorUsageHandler),
    });
    let function_fetcher = Arc::new(functions::function_fetcher::FetcherRouter::new(
        Arc::new(StubFunctionGithubFetcher),
        Arc::new(StubFilesystemFetcher),
        Arc::new(functions::function_fetcher::mock::MockFetcher),
    ));
    let profile_fetcher = Arc::new(functions::profile_fetcher::FetcherRouter::new(
        Arc::new(StubProfileGithubFetcher),
        Arc::new(StubFilesystemProfileFetcher),
        Arc::new(functions::profile_fetcher::mock::MockFetcher),
    ));
    Arc::new(super::Client::new(
        agent_client,
        ensemble_fetcher,
        vector_client,
        function_fetcher,
        profile_fetcher,
        Arc::new(StubFunctionUsageHandler),
    ))
}

fn make_request(
    function_repo: &str,
    profile_repo: &str,
    input: Input,
    seed: i64,
) -> Arc<Request> {
    Arc::new(Request::FunctionRemoteProfileRemote {
        path: FunctionRemoteProfileRemoteRequestPath {
            fremote: Remote::Mock,
            fowner: "mock".to_string(),
            frepository: function_repo.to_string(),
            fcommit: Some("mock".to_string()),
            premote: Remote::Mock,
            powner: "mock".to_string(),
            prepository: profile_repo.to_string(),
            pcommit: Some("mock".to_string()),
        },
        body: FunctionRemoteProfileRemoteRequestBody {
            retry_token: None,
            from_cache: None,
            reasoning: None,
            strategy: None,
            input,
            provider: None,
            seed: Some(seed),
            stream: None,
            mcp_server_authorization: None,
        },
    })
}

// ---------------------------------------------------------------------------
// Streaming + aggregation helpers
// ---------------------------------------------------------------------------

fn aggregate(chunks: Vec<FunctionExecutionChunk>) -> FunctionExecution {
    let mut agg: Option<FunctionExecutionChunk> = None;
    for chunk in &chunks {
        match &mut agg {
            Some(a) => a.push(chunk),
            None => agg = Some(chunk.clone()),
        }
    }
    FunctionExecution::from(agg.expect("stream should have at least one chunk"))
}

fn assert_chunk_invariants(chunks: &[FunctionExecutionChunk]) {
    assert!(!chunks.is_empty(), "stream must not be empty");
    for (i, chunk) in chunks.iter().enumerate() {
        assert!(
            chunk.tasks.len() <= 1,
            "chunk {i} has {} tasks, expected at most 1",
            chunk.tasks.len(),
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
}

async fn run_execution(client: &Arc<TestClient>, request: Arc<Request>) -> FunctionExecution {
    let ctx = ctx::Context::new(Arc::new(ctx::DefaultContextExt), Decimal::ONE);
    let stream = client
        .clone()
        .create_streaming(ctx, request)
        .await
        .expect("create_streaming should succeed");
    let chunks: Vec<_> = Box::pin(stream).collect().await;
    assert_chunk_invariants(&chunks);
    aggregate(chunks)
}

// ---------------------------------------------------------------------------
// Snapshot helpers
// ---------------------------------------------------------------------------

fn normalize(mut fe: FunctionExecution) -> FunctionExecution {
    normalize_fe(&mut fe);
    fe
}

fn normalize_fe(fe: &mut FunctionExecution) {
    fe.id = String::new();
    fe.created = 0;
    fe.retry_token = None;
    for task in &mut fe.tasks {
        match task {
            objectiveai::functions::executions::response::unary::Task::VectorCompletion(vt) => {
                normalize_vc(&mut vt.inner);
            }
            objectiveai::functions::executions::response::unary::Task::FunctionExecution(ft) => {
                normalize_fe(&mut ft.inner);
            }
        }
    }
}

fn normalize_vc(
    vc: &mut objectiveai::vector::completions::response::unary::VectorCompletion,
) {
    vc.id = String::new();
    vc.created = 0;
    for completion in &mut vc.completions {
        completion.inner.id = String::new();
        completion.inner.created = 0;
        for msg in &mut completion.inner.messages {
            if let objectiveai::agent::completions::response::unary::Message::Assistant(asst) = msg
            {
                asst.upstream_id = String::new();
                asst.created = 0;
            }
        }
    }
    for vote in &mut vc.votes {
        vote.prompt_id = String::new();
        vote.responses_ids = Vec::new();
    }
}

fn assert_snapshot(json: &str, path: &str, expected: &str) {
    if std::env::var("UPDATE_FUNCTIONS_EXECUTIONS_CLIENT_TESTS_SNAPSHOTS").as_deref() == Ok("1") {
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

/// mock-1: Simple scalar leaf, single task, binary classification, seed 42.
#[tokio::test]
async fn test_mock_1_scalar_leaf_binary_seed_42() {
    let client = make_client();
    let request = make_request(
        "mock-1",
        "mock-1",
        Input::Object(indexmap::indexmap! {
            "text".into() => Input::String("Hello world".into()),
        }),
        42,
    );
    let result = normalize(run_execution(&client, request).await);
    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/executions/client_tests/mock_1_scalar_leaf_binary_seed_42.json"),
        include_str!("../../../assets/functions/executions/client_tests/mock_1_scalar_leaf_binary_seed_42.json"),
    );
}

/// mock-2: Multi-task scalar with skip condition (include_sentiment=false), seed 42.
#[tokio::test]
async fn test_mock_2_scalar_skip_false_seed_42() {
    let client = make_client();
    let request = make_request(
        "mock-2",
        "mock-2",
        Input::Object(indexmap::indexmap! {
            "text".into() => Input::String("Buy cheap watches now!!!".into()),
            "include_sentiment".into() => Input::Boolean(false),
        }),
        42,
    );
    let result = normalize(run_execution(&client, request).await);
    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/executions/client_tests/mock_2_scalar_skip_false_seed_42.json"),
        include_str!("../../../assets/functions/executions/client_tests/mock_2_scalar_skip_false_seed_42.json"),
    );
}

/// mock-2: Multi-task scalar with skip condition (include_sentiment=true), seed 42.
#[tokio::test]
async fn test_mock_2_scalar_skip_true_seed_42() {
    let client = make_client();
    let request = make_request(
        "mock-2",
        "mock-2",
        Input::Object(indexmap::indexmap! {
            "text".into() => Input::String("I love this product!".into()),
            "include_sentiment".into() => Input::Boolean(true),
        }),
        42,
    );
    let result = normalize(run_execution(&client, request).await);
    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/executions/client_tests/mock_2_scalar_skip_true_seed_42.json"),
        include_str!("../../../assets/functions/executions/client_tests/mock_2_scalar_skip_true_seed_42.json"),
    );
}

/// mock-3: 5-way classification scalar, seed 42.
#[tokio::test]
async fn test_mock_3_scalar_5way_seed_42() {
    let client = make_client();
    let request = make_request(
        "mock-3",
        "mock-3",
        Input::Object(indexmap::indexmap! {
            "text".into() => Input::String("The food was amazing".into()),
        }),
        42,
    );
    let result = normalize(run_execution(&client, request).await);
    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/executions/client_tests/mock_3_scalar_5way_seed_42.json"),
        include_str!("../../../assets/functions/executions/client_tests/mock_3_scalar_5way_seed_42.json"),
    );
}

/// mock-4: Simple vector ranker with 3 items, seed 42.
#[tokio::test]
async fn test_mock_4_vector_ranker_seed_42() {
    let client = make_client();
    let request = make_request(
        "mock-4",
        "mock-4",
        Input::Object(indexmap::indexmap! {
            "items".into() => Input::Array(vec![
                Input::String("Apple".into()),
                Input::String("Banana".into()),
                Input::String("Cherry".into()),
            ]),
        }),
        42,
    );
    let result = normalize(run_execution(&client, request).await);
    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/executions/client_tests/mock_4_vector_ranker_seed_42.json"),
        include_str!("../../../assets/functions/executions/client_tests/mock_4_vector_ranker_seed_42.json"),
    );
}

/// mock-5: Vector ranker with context and multiple tasks, seed 42.
#[tokio::test]
async fn test_mock_5_vector_context_multi_task_seed_42() {
    let client = make_client();
    let request = make_request(
        "mock-5",
        "mock-5",
        Input::Object(indexmap::indexmap! {
            "context".into() => Input::Object(indexmap::indexmap! {
                "query".into() => Input::String("best fruit".into()),
            }),
            "items".into() => Input::Array(vec![
                Input::String("Apple".into()),
                Input::String("Banana".into()),
            ]),
        }),
        42,
    );
    let result = normalize(run_execution(&client, request).await);
    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/executions/client_tests/mock_5_vector_context_multi_task_seed_42.json"),
        include_str!("../../../assets/functions/executions/client_tests/mock_5_vector_context_multi_task_seed_42.json"),
    );
}

/// mock-6: Scalar with system message and multi-part user content, seed 42.
#[tokio::test]
async fn test_mock_6_scalar_system_message_seed_42() {
    let client = make_client();
    let request = make_request(
        "mock-6",
        "mock-6",
        Input::Object(indexmap::indexmap! {
            "subject".into() => Input::String("Meeting tomorrow".into()),
            "body".into() => Input::String("Don't forget the meeting at 3pm.".into()),
        }),
        42,
    );
    let result = normalize(run_execution(&client, request).await);
    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/executions/client_tests/mock_6_scalar_system_message_seed_42.json"),
        include_str!("../../../assets/functions/executions/client_tests/mock_6_scalar_system_message_seed_42.json"),
    );
}

// ---------------------------------------------------------------------------
// Vector leaf with 5 tasks
// ---------------------------------------------------------------------------

/// mock-7: Vector ranker with 5 scoring criteria, seed 42.
#[tokio::test]
async fn test_mock_7_vector_5_criteria_seed_42() {
    let client = make_client();
    let request = make_request(
        "mock-7",
        "mock-7",
        Input::Object(indexmap::indexmap! {
            "items".into() => Input::Array(vec![
                Input::String("Option A".into()),
                Input::String("Option B".into()),
                Input::String("Option C".into()),
            ]),
        }),
        42,
    );
    let result = normalize(run_execution(&client, request).await);
    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/executions/client_tests/mock_7_vector_5_criteria_seed_42.json"),
        include_str!("../../../assets/functions/executions/client_tests/mock_7_vector_5_criteria_seed_42.json"),
    );
}

/// mock-8: Vector ranker with context, 5 tasks, skip conditions (strict=false), seed 42.
#[tokio::test]
async fn test_mock_8_vector_context_skip_false_seed_42() {
    let client = make_client();
    let request = make_request(
        "mock-8",
        "mock-8",
        Input::Object(indexmap::indexmap! {
            "context".into() => Input::Object(indexmap::indexmap! {
                "query".into() => Input::String("best answer".into()),
                "strict".into() => Input::Boolean(false),
            }),
            "items".into() => Input::Array(vec![
                Input::String("Answer 1".into()),
                Input::String("Answer 2".into()),
            ]),
        }),
        42,
    );
    let result = normalize(run_execution(&client, request).await);
    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/executions/client_tests/mock_8_vector_context_skip_false_seed_42.json"),
        include_str!("../../../assets/functions/executions/client_tests/mock_8_vector_context_skip_false_seed_42.json"),
    );
}

/// mock-8: Vector ranker with context, 5 tasks, skip conditions (strict=true), seed 42.
#[tokio::test]
async fn test_mock_8_vector_context_skip_true_seed_42() {
    let client = make_client();
    let request = make_request(
        "mock-8",
        "mock-8",
        Input::Object(indexmap::indexmap! {
            "context".into() => Input::Object(indexmap::indexmap! {
                "query".into() => Input::String("best answer".into()),
                "strict".into() => Input::Boolean(true),
            }),
            "items".into() => Input::Array(vec![
                Input::String("Answer 1".into()),
                Input::String("Answer 2".into()),
            ]),
        }),
        42,
    );
    let result = normalize(run_execution(&client, request).await);
    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/executions/client_tests/mock_8_vector_context_skip_true_seed_42.json"),
        include_str!("../../../assets/functions/executions/client_tests/mock_8_vector_context_skip_true_seed_42.json"),
    );
}

// ---------------------------------------------------------------------------
// Scalar branch functions
// ---------------------------------------------------------------------------

/// mock-9: Scalar branch combining spam + importance classifiers, seed 42.
#[tokio::test]
async fn test_mock_9_scalar_branch_2_tasks_seed_42() {
    let client = make_client();
    let request = make_request(
        "mock-9",
        "mock-9",
        Input::Object(indexmap::indexmap! {
            "text".into() => Input::String("Important project update".into()),
            "subject".into() => Input::String("Project update".into()),
        }),
        42,
    );
    let result = normalize(run_execution(&client, request).await);
    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/executions/client_tests/mock_9_scalar_branch_2_tasks_seed_42.json"),
        include_str!("../../../assets/functions/executions/client_tests/mock_9_scalar_branch_2_tasks_seed_42.json"),
    );
}

/// mock-10: Scalar branch combining binary, 5-way, importance (one agent errors), seed 42.
#[tokio::test]
async fn test_mock_10_scalar_branch_3_tasks_error_seed_42() {
    let client = make_client();
    let request = make_request(
        "mock-10",
        "mock-10",
        Input::Object(indexmap::indexmap! {
            "text".into() => Input::String("Great service!".into()),
        }),
        42,
    );
    let result = normalize(run_execution(&client, request).await);
    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/executions/client_tests/mock_10_scalar_branch_3_tasks_error_seed_42.json"),
        include_str!("../../../assets/functions/executions/client_tests/mock_10_scalar_branch_3_tasks_error_seed_42.json"),
    );
}

/// mock-11: Scalar branch with skip condition (include_sentiment=false), seed 42.
#[tokio::test]
async fn test_mock_11_scalar_branch_skip_false_seed_42() {
    let client = make_client();
    let request = make_request(
        "mock-11",
        "mock-11",
        Input::Object(indexmap::indexmap! {
            "text".into() => Input::String("Check this out".into()),
            "include_sentiment".into() => Input::Boolean(false),
        }),
        42,
    );
    let result = normalize(run_execution(&client, request).await);
    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/executions/client_tests/mock_11_scalar_branch_skip_false_seed_42.json"),
        include_str!("../../../assets/functions/executions/client_tests/mock_11_scalar_branch_skip_false_seed_42.json"),
    );
}

/// mock-11: Scalar branch with skip condition (include_sentiment=true), seed 42.
#[tokio::test]
async fn test_mock_11_scalar_branch_skip_true_seed_42() {
    let client = make_client();
    let request = make_request(
        "mock-11",
        "mock-11",
        Input::Object(indexmap::indexmap! {
            "text".into() => Input::String("Check this out".into()),
            "include_sentiment".into() => Input::Boolean(true),
        }),
        42,
    );
    let result = normalize(run_execution(&client, request).await);
    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/executions/client_tests/mock_11_scalar_branch_skip_true_seed_42.json"),
        include_str!("../../../assets/functions/executions/client_tests/mock_11_scalar_branch_skip_true_seed_42.json"),
    );
}

// ---------------------------------------------------------------------------
// Vector branch functions
// ---------------------------------------------------------------------------

/// mock-12: Vector branch with two vector sub-function rankers, seed 42.
#[tokio::test]
async fn test_mock_12_vector_branch_2_vector_seed_42() {
    let client = make_client();
    let request = make_request(
        "mock-12",
        "mock-12",
        Input::Object(indexmap::indexmap! {
            "items".into() => Input::Array(vec![
                Input::String("Red".into()),
                Input::String("Blue".into()),
                Input::String("Green".into()),
            ]),
        }),
        42,
    );
    let result = normalize(run_execution(&client, request).await);
    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/executions/client_tests/mock_12_vector_branch_2_vector_seed_42.json"),
        include_str!("../../../assets/functions/executions/client_tests/mock_12_vector_branch_2_vector_seed_42.json"),
    );
}

/// mock-13: Vector branch mixing scalar and vector sub-functions, seed 42.
#[tokio::test]
async fn test_mock_13_vector_branch_mixed_seed_42() {
    let client = make_client();
    let request = make_request(
        "mock-13",
        "mock-13",
        Input::Object(indexmap::indexmap! {
            "context".into() => Input::Object(indexmap::indexmap! {
                "query".into() => Input::String("favorite color".into()),
            }),
            "items".into() => Input::Array(vec![
                Input::String("Red".into()),
                Input::String("Blue".into()),
            ]),
        }),
        42,
    );
    let result = normalize(run_execution(&client, request).await);
    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/executions/client_tests/mock_13_vector_branch_mixed_seed_42.json"),
        include_str!("../../../assets/functions/executions/client_tests/mock_13_vector_branch_mixed_seed_42.json"),
    );
}

/// mock-14: Vector branch with skip on sub-function (include_quality=false), seed 42.
#[tokio::test]
async fn test_mock_14_vector_branch_skip_false_seed_42() {
    let client = make_client();
    let request = make_request(
        "mock-14",
        "mock-14",
        Input::Object(indexmap::indexmap! {
            "context".into() => Input::Object(indexmap::indexmap! {
                "query".into() => Input::String("rank these".into()),
                "include_quality".into() => Input::Boolean(false),
            }),
            "items".into() => Input::Array(vec![
                Input::String("X".into()),
                Input::String("Y".into()),
            ]),
        }),
        42,
    );
    let result = normalize(run_execution(&client, request).await);
    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/executions/client_tests/mock_14_vector_branch_skip_false_seed_42.json"),
        include_str!("../../../assets/functions/executions/client_tests/mock_14_vector_branch_skip_false_seed_42.json"),
    );
}

/// mock-14: Vector branch with skip on sub-function (include_quality=true), seed 42.
#[tokio::test]
async fn test_mock_14_vector_branch_skip_true_seed_42() {
    let client = make_client();
    let request = make_request(
        "mock-14",
        "mock-14",
        Input::Object(indexmap::indexmap! {
            "context".into() => Input::Object(indexmap::indexmap! {
                "query".into() => Input::String("rank these".into()),
                "include_quality".into() => Input::Boolean(true),
            }),
            "items".into() => Input::Array(vec![
                Input::String("X".into()),
                Input::String("Y".into()),
            ]),
        }),
        42,
    );
    let result = normalize(run_execution(&client, request).await);
    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/executions/client_tests/mock_14_vector_branch_skip_true_seed_42.json"),
        include_str!("../../../assets/functions/executions/client_tests/mock_14_vector_branch_skip_true_seed_42.json"),
    );
}

/// mock-15: Vector branch with 3 vector sub-functions and high logprobs, seed 42.
#[tokio::test]
async fn test_mock_15_vector_branch_3_vector_logprobs_seed_42() {
    let client = make_client();
    let request = make_request(
        "mock-15",
        "mock-15",
        Input::Object(indexmap::indexmap! {
            "items".into() => Input::Array(vec![
                Input::String("Alpha".into()),
                Input::String("Beta".into()),
            ]),
        }),
        42,
    );
    let result = normalize(run_execution(&client, request).await);
    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/executions/client_tests/mock_15_vector_branch_3_vector_logprobs_seed_42.json"),
        include_str!("../../../assets/functions/executions/client_tests/mock_15_vector_branch_3_vector_logprobs_seed_42.json"),
    );
}

/// mock-16: Vector branch with 4 tasks, error agent, logprobs, seed 42.
#[tokio::test]
async fn test_mock_16_vector_branch_4_tasks_error_logprobs_seed_42() {
    let client = make_client();
    let request = make_request(
        "mock-16",
        "mock-16",
        Input::Object(indexmap::indexmap! {
            "context".into() => Input::Object(indexmap::indexmap! {
                "text".into() => Input::String("Evaluate these options".into()),
            }),
            "items".into() => Input::Array(vec![
                Input::String("First".into()),
                Input::String("Second".into()),
                Input::String("Third".into()),
            ]),
        }),
        42,
    );
    let result = normalize(run_execution(&client, request).await);
    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/executions/client_tests/mock_16_vector_branch_4_tasks_error_logprobs_seed_42.json"),
        include_str!("../../../assets/functions/executions/client_tests/mock_16_vector_branch_4_tasks_error_logprobs_seed_42.json"),
    );
}

/// mock-17: Vector branch with mixed tasks, skip conditions (deep=false), seed 42.
#[tokio::test]
async fn test_mock_17_vector_branch_mixed_skip_false_seed_42() {
    let client = make_client();
    let request = make_request(
        "mock-17",
        "mock-17",
        Input::Object(indexmap::indexmap! {
            "context".into() => Input::Object(indexmap::indexmap! {
                "query".into() => Input::String("compare these".into()),
                "deep".into() => Input::Boolean(false),
            }),
            "items".into() => Input::Array(vec![
                Input::String("Foo".into()),
                Input::String("Bar".into()),
            ]),
        }),
        42,
    );
    let result = normalize(run_execution(&client, request).await);
    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/executions/client_tests/mock_17_vector_branch_mixed_skip_false_seed_42.json"),
        include_str!("../../../assets/functions/executions/client_tests/mock_17_vector_branch_mixed_skip_false_seed_42.json"),
    );
}

/// mock-17: Vector branch with mixed tasks, skip conditions (deep=true), seed 42.
#[tokio::test]
async fn test_mock_17_vector_branch_mixed_skip_true_seed_42() {
    let client = make_client();
    let request = make_request(
        "mock-17",
        "mock-17",
        Input::Object(indexmap::indexmap! {
            "context".into() => Input::Object(indexmap::indexmap! {
                "query".into() => Input::String("compare these".into()),
                "deep".into() => Input::Boolean(true),
            }),
            "items".into() => Input::Array(vec![
                Input::String("Foo".into()),
                Input::String("Bar".into()),
            ]),
        }),
        42,
    );
    let result = normalize(run_execution(&client, request).await);
    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/executions/client_tests/mock_17_vector_branch_mixed_skip_true_seed_42.json"),
        include_str!("../../../assets/functions/executions/client_tests/mock_17_vector_branch_mixed_skip_true_seed_42.json"),
    );
}

// ---------------------------------------------------------------------------
// Super branch tests (branch functions whose tasks are branch functions)
// ---------------------------------------------------------------------------

/// mock-18: Scalar super branch, 2 scalar branch sub-functions, seed 42.
#[tokio::test]
async fn test_mock_18_scalar_super_branch_seed_42() {
    let client = make_client();
    let request = make_request(
        "mock-18",
        "mock-18",
        Input::Object(indexmap::indexmap! {
            "text".into() => Input::String("Hello world".into()),
            "subject".into() => Input::String("greeting".into()),
        }),
        42,
    );
    let result = normalize(run_execution(&client, request).await);
    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/executions/client_tests/mock_18_scalar_super_branch_seed_42.json"),
        include_str!("../../../assets/functions/executions/client_tests/mock_18_scalar_super_branch_seed_42.json"),
    );
}

/// mock-19: Scalar super branch with skip (thorough=false), seed 42.
#[tokio::test]
async fn test_mock_19_scalar_super_branch_skip_false_seed_42() {
    let client = make_client();
    let request = make_request(
        "mock-19",
        "mock-19",
        Input::Object(indexmap::indexmap! {
            "text".into() => Input::String("Test input".into()),
            "subject".into() => Input::String("testing".into()),
            "thorough".into() => Input::Boolean(false),
        }),
        42,
    );
    let result = normalize(run_execution(&client, request).await);
    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/executions/client_tests/mock_19_scalar_super_branch_skip_false_seed_42.json"),
        include_str!("../../../assets/functions/executions/client_tests/mock_19_scalar_super_branch_skip_false_seed_42.json"),
    );
}

/// mock-19: Scalar super branch with skip (thorough=true), seed 42.
#[tokio::test]
async fn test_mock_19_scalar_super_branch_skip_true_seed_42() {
    let client = make_client();
    let request = make_request(
        "mock-19",
        "mock-19",
        Input::Object(indexmap::indexmap! {
            "text".into() => Input::String("Test input".into()),
            "subject".into() => Input::String("testing".into()),
            "thorough".into() => Input::Boolean(true),
        }),
        42,
    );
    let result = normalize(run_execution(&client, request).await);
    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/executions/client_tests/mock_19_scalar_super_branch_skip_true_seed_42.json"),
        include_str!("../../../assets/functions/executions/client_tests/mock_19_scalar_super_branch_skip_true_seed_42.json"),
    );
}

/// mock-20: Vector super branch, 2 vector branch sub-functions, seed 42.
#[tokio::test]
async fn test_mock_20_vector_super_branch_seed_42() {
    let client = make_client();
    let request = make_request(
        "mock-20",
        "mock-20",
        Input::Object(indexmap::indexmap! {
            "items".into() => Input::Array(vec![
                Input::String("Alpha".into()),
                Input::String("Beta".into()),
                Input::String("Gamma".into()),
            ]),
        }),
        42,
    );
    let result = normalize(run_execution(&client, request).await);
    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/executions/client_tests/mock_20_vector_super_branch_seed_42.json"),
        include_str!("../../../assets/functions/executions/client_tests/mock_20_vector_super_branch_seed_42.json"),
    );
}

/// mock-21: Vector super branch with context, 3 vector branch sub-functions, seed 42.
#[tokio::test]
async fn test_mock_21_vector_super_branch_context_seed_42() {
    let client = make_client();
    let request = make_request(
        "mock-21",
        "mock-21",
        Input::Object(indexmap::indexmap! {
            "context".into() => Input::Object(indexmap::indexmap! {
                "text".into() => Input::String("rank these options".into()),
            }),
            "items".into() => Input::Array(vec![
                Input::String("One".into()),
                Input::String("Two".into()),
            ]),
        }),
        42,
    );
    let result = normalize(run_execution(&client, request).await);
    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/executions/client_tests/mock_21_vector_super_branch_context_seed_42.json"),
        include_str!("../../../assets/functions/executions/client_tests/mock_21_vector_super_branch_context_seed_42.json"),
    );
}

// ===========================================================================
// Error tests
// ===========================================================================

/// Helper: create a request with custom body fields.
fn make_request_with_body(
    function_repo: &str,
    profile_repo: &str,
    body: FunctionRemoteProfileRemoteRequestBody,
) -> Arc<Request> {
    Arc::new(Request::FunctionRemoteProfileRemote {
        path: FunctionRemoteProfileRemoteRequestPath {
            fremote: Remote::Mock,
            fowner: "mock".to_string(),
            frepository: function_repo.to_string(),
            fcommit: Some("mock".to_string()),
            premote: Remote::Mock,
            powner: "mock".to_string(),
            prepository: profile_repo.to_string(),
            pcommit: Some("mock".to_string()),
        },
        body,
    })
}

/// Helper: expect create_streaming to return Err with a specific status code.
async fn expect_err(client: &Arc<TestClient>, request: Arc<Request>, expected_status: u16) -> super::Error {
    let ctx = ctx::Context::new(Arc::new(ctx::DefaultContextExt), Decimal::ONE);
    match client.clone().create_streaming(ctx, request).await {
        Ok(_) => panic!("expected create_streaming to fail, but it succeeded"),
        Err(err) => {
            assert_eq!(err.status(), expected_status, "error: {err}");
            err
        }
    }
}

/// Helper: run execution and return the aggregated result (for tests where
/// the stream succeeds but the response contains error fields).
async fn run_execution_allow_error(client: &Arc<TestClient>, request: Arc<Request>) -> FunctionExecution {
    let ctx = ctx::Context::new(Arc::new(ctx::DefaultContextExt), Decimal::ONE);
    let stream = client
        .clone()
        .create_streaming(ctx, request)
        .await
        .expect("create_streaming should succeed");
    let chunks: Vec<_> = Box::pin(stream).collect().await;
    assert_chunk_invariants(&chunks);
    aggregate(chunks)
}

// ---------------------------------------------------------------------------
// 1. Pre-Execution Errors
// ---------------------------------------------------------------------------

/// 1.1: InvalidRetryToken — garbage retry_token string.
#[tokio::test]
async fn test_error_1_1_invalid_retry_token() {
    let client = make_client();
    let request = make_request_with_body(
        "mock-1",
        "mock-1",
        FunctionRemoteProfileRemoteRequestBody {
            retry_token: Some("not-a-valid-retry-token!!!".to_string()),
            from_cache: None,
            reasoning: None,
            strategy: None,
            input: Input::Object(indexmap::indexmap! {
                "text".into() => Input::String("test".into()),
            }),
            provider: None,
            seed: Some(42),
            stream: None,
            mcp_server_authorization: None,
        },
    );
    let err = expect_err(&client, request, 400).await;
    assert!(matches!(err, super::Error::InvalidRetryToken), "expected InvalidRetryToken, got: {err}");
}

/// 1.3: InvalidFunctionForStrategy — scalar function with Swiss strategy.
#[tokio::test]
async fn test_error_1_3_scalar_function_swiss_strategy() {
    let client = make_client();
    let request = make_request_with_body(
        "mock-1",
        "mock-1",
        FunctionRemoteProfileRemoteRequestBody {
            retry_token: None,
            from_cache: None,
            reasoning: None,
            strategy: Some(Strategy::SwissSystem { pool: None, rounds: None }),
            input: Input::Object(indexmap::indexmap! {
                "text".into() => Input::String("test".into()),
            }),
            provider: None,
            seed: Some(42),
            stream: None,
            mcp_server_authorization: None,
        },
    );
    let err = expect_err(&client, request, 400).await;
    assert!(matches!(err, super::Error::InvalidFunctionForStrategy(_)), "expected InvalidFunctionForStrategy, got: {err}");
}

/// 1.4: InvalidStrategy — Swiss strategy with pool=1.
#[tokio::test]
async fn test_error_1_4_invalid_strategy_pool() {
    let client = make_client();
    let request = make_request_with_body(
        "mock-4",
        "mock-4",
        FunctionRemoteProfileRemoteRequestBody {
            retry_token: None,
            from_cache: None,
            reasoning: None,
            strategy: Some(Strategy::SwissSystem { pool: Some(1), rounds: Some(3) }),
            input: Input::Object(indexmap::indexmap! {
                "items".into() => Input::Array(vec![
                    Input::String("A".into()),
                    Input::String("B".into()),
                    Input::String("C".into()),
                ]),
            }),
            provider: None,
            seed: Some(42),
            stream: None,
            mcp_server_authorization: None,
        },
    );
    let err = expect_err(&client, request, 400).await;
    assert!(matches!(err, super::Error::InvalidStrategy(_)), "expected InvalidStrategy, got: {err}");
}

// ---------------------------------------------------------------------------
// 2. Flat Task Profile Fetch Errors
// ---------------------------------------------------------------------------

/// 2.1: FunctionNotFound — non-existent mock function repository.
#[tokio::test]
async fn test_error_2_1_function_not_found() {
    let client = make_client();
    let request = make_request("mock-nonexistent", "mock-1", Input::Object(indexmap::indexmap! {}), 42);
    let err = expect_err(&client, request, 404).await;
    assert!(matches!(err, super::Error::FunctionNotFound), "expected FunctionNotFound, got: {err}");
}

/// 2.3: ProfileNotFound — non-existent mock profile repository.
#[tokio::test]
async fn test_error_2_3_profile_not_found() {
    let client = make_client();
    let request = make_request(
        "mock-1",
        "mock-nonexistent",
        Input::Object(indexmap::indexmap! {
            "text".into() => Input::String("test".into()),
        }),
        42,
    );
    let err = expect_err(&client, request, 404).await;
    assert!(matches!(err, super::Error::ProfileNotFound), "expected ProfileNotFound, got: {err}");
}

/// 2.5: InputSchemaMismatch — wrong input shape for mock-1.
#[tokio::test]
async fn test_error_2_5_input_schema_mismatch() {
    let client = make_client();
    let request = make_request(
        "mock-1",
        "mock-1",
        Input::Object(indexmap::indexmap! {
            "wrong_field".into() => Input::String("test".into()),
        }),
        42,
    );
    let err = expect_err(&client, request, 400).await;
    assert!(matches!(err, super::Error::InputSchemaMismatch), "expected InputSchemaMismatch, got: {err}");
}

/// 2.6: InvalidProfile — tasks length mismatch (2 task profiles for 1-task function).
#[tokio::test]
async fn test_error_2_6_tasks_length_mismatch() {
    let client = make_client();
    let request = make_request(
        "mock-1",
        "mock-err-11",
        Input::Object(indexmap::indexmap! {
            "text".into() => Input::String("test".into()),
        }),
        42,
    );
    let err = expect_err(&client, request, 400).await;
    assert!(matches!(err, super::Error::InvalidProfile(_)), "expected InvalidProfile, got: {err}");
}

/// 2.7: InvalidProfile — weights length mismatch (2 weights for 1-task function).
#[tokio::test]
async fn test_error_2_7_weights_length_mismatch() {
    let client = make_client();
    let request = make_request(
        "mock-1",
        "mock-err-12",
        Input::Object(indexmap::indexmap! {
            "text".into() => Input::String("test".into()),
        }),
        42,
    );
    let err = expect_err(&client, request, 400).await;
    assert!(matches!(err, super::Error::InvalidProfile(_)), "expected InvalidProfile, got: {err}");
}

/// 2.8: InvalidProfile — placeholder for function task.
#[tokio::test]
async fn test_error_2_8_placeholder_for_function_task() {
    let client = make_client();
    let request = make_request(
        "mock-9",
        "mock-err-13",
        Input::Object(indexmap::indexmap! {
            "text".into() => Input::String("test".into()),
            "subject".into() => Input::String("test".into()),
        }),
        42,
    );
    let err = expect_err(&client, request, 400).await;
    assert!(matches!(err, super::Error::InvalidProfile(_)), "expected InvalidProfile, got: {err}");
}

/// 2.9: InvalidProfile — Remote profile for VC task.
#[tokio::test]
async fn test_error_2_9_remote_for_vc_task() {
    let client = make_client();
    let request = make_request(
        "mock-1",
        "mock-err-14",
        Input::Object(indexmap::indexmap! {
            "text".into() => Input::String("test".into()),
        }),
        42,
    );
    let err = expect_err(&client, request, 400).await;
    assert!(matches!(err, super::Error::InvalidProfile(_)), "expected InvalidProfile, got: {err}");
}

/// 2.17: InvalidAppExpression — task expression references missing key.
#[tokio::test]
async fn test_error_2_17_bad_task_expression() {
    let client = make_client();
    let request = make_request(
        "mock-err-8",
        "mock-err-8",
        Input::Object(indexmap::indexmap! {
            "text".into() => Input::String("test".into()),
        }),
        42,
    );
    let err = expect_err(&client, request, 400).await;
    assert!(matches!(err, super::Error::InvalidAppExpression(_)), "expected InvalidAppExpression, got: {err}");
}

/// 2.19: FetchEnsemble — string ensemble ID hits StubEnsembleFetcher (returns 501).
#[tokio::test]
async fn test_error_2_19_fetch_ensemble() {
    let client = make_client();
    let request = make_request(
        "mock-1",
        "mock-err-15",
        Input::Object(indexmap::indexmap! {
            "text".into() => Input::String("test".into()),
        }),
        42,
    );
    let err = expect_err(&client, request, 501).await;
    assert!(matches!(err, super::Error::FetchEnsemble(_)), "expected FetchEnsemble, got: {err}");
}

/// 2.20: InvalidEnsemble — 1 agent but 2 profile weights.
#[tokio::test]
async fn test_error_2_20_invalid_ensemble() {
    let client = make_client();
    let request = make_request(
        "mock-1",
        "mock-err-16",
        Input::Object(indexmap::indexmap! {
            "text".into() => Input::String("test".into()),
        }),
        42,
    );
    let err = expect_err(&client, request, 400).await;
    assert!(matches!(err, super::Error::InvalidEnsemble(_)), "expected InvalidEnsemble, got: {err}");
}

/// 2.21: Recursive FunctionNotFound — branch references mock-999.
#[tokio::test]
async fn test_error_2_21_recursive_function_not_found() {
    let client = make_client();
    let request = make_request(
        "mock-err-9",
        "mock-err-9",
        Input::Object(indexmap::indexmap! {
            "text".into() => Input::String("test".into()),
        }),
        42,
    );
    let err = expect_err(&client, request, 404).await;
    assert!(matches!(err, super::Error::FunctionNotFound), "expected FunctionNotFound, got: {err}");
}

/// 2.22: Recursive ProfileNotFound — tasks profile references mock-999.
#[tokio::test]
async fn test_error_2_22_recursive_profile_not_found() {
    let client = make_client();
    let request = make_request(
        "mock-9",
        "mock-err-17",
        Input::Object(indexmap::indexmap! {
            "text".into() => Input::String("test".into()),
            "subject".into() => Input::String("test".into()),
        }),
        42,
    );
    let err = expect_err(&client, request, 404).await;
    assert!(matches!(err, super::Error::ProfileNotFound), "expected ProfileNotFound, got: {err}");
}

/// 2.23: Recursive InputSchemaMismatch — wrong input for sub-function.
#[tokio::test]
async fn test_error_2_23_recursive_input_schema_mismatch() {
    let client = make_client();
    let request = make_request(
        "mock-err-10",
        "mock-err-10",
        Input::Object(indexmap::indexmap! {
            "text".into() => Input::String("test".into()),
        }),
        42,
    );
    let err = expect_err(&client, request, 400).await;
    assert!(matches!(err, super::Error::InputSchemaMismatch), "expected InputSchemaMismatch, got: {err}");
}

// ---------------------------------------------------------------------------
// 3. Vector Completion Errors (execution-time)
// ---------------------------------------------------------------------------

/// 3.1: All agents error — VC agents fail, completions have error finish_reason,
/// output is fallback uniform → weighted sum to 0.5.
#[tokio::test]
async fn test_error_3_1_all_agents_error() {
    let client = make_client();
    let request = make_request(
        "mock-1",
        "mock-err-18",
        Input::Object(indexmap::indexmap! {
            "text".into() => Input::String("test".into()),
        }),
        42,
    );
    let result = run_execution_allow_error(&client, request).await;
    assert_eq!(result.tasks.len(), 1);
    match &result.tasks[0] {
        objectiveai::functions::executions::response::unary::Task::VectorCompletion(vt) => {
            // The task itself should not have an error (VC "succeeds" with fallback).
            assert!(vt.error.is_none(), "expected no task-level error, got: {:?}", vt.error);
            assert!(!vt.inner.completions.is_empty(), "expected at least one completion");
            for completion in &vt.inner.completions {
                // Each agent completion should have an error set.
                assert!(
                    completion.inner.error.is_some(),
                    "expected error on agent completion, got None",
                );
            }
        }
        other => panic!("expected VectorCompletion task, got: {other:?}"),
    }
    // Output is the fallback weighted sum of uniform distribution.
    assert!(
        matches!(&result.output, objectiveai::functions::expression::TaskOutputOwned::Scalar(s) if *s == rust_decimal::dec!(0.5)),
        "expected Scalar(0.5) fallback, got: {:?}",
        result.output,
    );
}

// ---------------------------------------------------------------------------
// 4. Task Output Expression Errors (execution-time)
// ---------------------------------------------------------------------------

/// 4.1: Output expression evaluation fails (references nonexistent field).
#[tokio::test]
async fn test_error_4_1_output_expression_fails() {
    let client = make_client();
    let request = make_request(
        "mock-err-1",
        "mock-err-1",
        Input::Object(indexmap::indexmap! {
            "text".into() => Input::String("test".into()),
        }),
        42,
    );
    let result = run_execution_allow_error(&client, request).await;
    assert!(result.error.is_some(), "expected error on response");
    assert!(
        matches!(result.output, objectiveai::functions::expression::TaskOutputOwned::Err(_)),
        "expected Err output, got: {:?}",
        result.output,
    );
}

/// 4.2: Scalar output out of range (returns -1.0).
#[tokio::test]
async fn test_error_4_2_scalar_output_out_of_range() {
    let client = make_client();
    let request = make_request(
        "mock-err-2",
        "mock-err-2",
        Input::Object(indexmap::indexmap! {
            "text".into() => Input::String("test".into()),
        }),
        42,
    );
    let result = run_execution_allow_error(&client, request).await;
    assert!(result.error.is_some(), "expected error on response");
    assert!(
        matches!(result.output, objectiveai::functions::expression::TaskOutputOwned::Err(_)),
        "expected Err output, got: {:?}",
        result.output,
    );
}

/// 4.3: Scalar function got vector output.
#[tokio::test]
async fn test_error_4_3_scalar_got_vector() {
    let client = make_client();
    let request = make_request(
        "mock-err-3",
        "mock-err-3",
        Input::Object(indexmap::indexmap! {
            "text".into() => Input::String("test".into()),
        }),
        42,
    );
    let result = run_execution_allow_error(&client, request).await;
    assert!(result.error.is_some(), "expected error on response");
    assert!(
        matches!(result.output, objectiveai::functions::expression::TaskOutputOwned::Err(_)),
        "expected Err output, got: {:?}",
        result.output,
    );
}

/// 4.4: Vector output bad sum (scores doubled).
#[tokio::test]
async fn test_error_4_4_vector_output_bad_sum() {
    let client = make_client();
    let request = make_request(
        "mock-err-4",
        "mock-err-4",
        Input::Object(indexmap::indexmap! {
            "items".into() => Input::Array(vec![
                Input::String("A".into()),
                Input::String("B".into()),
            ]),
        }),
        42,
    );
    let result = run_execution_allow_error(&client, request).await;
    assert!(result.error.is_some(), "expected error on response");
    assert!(
        matches!(result.output, objectiveai::functions::expression::TaskOutputOwned::Err(_)),
        "expected Err output, got: {:?}",
        result.output,
    );
}

/// 4.5: Vector function got scalar output.
#[tokio::test]
async fn test_error_4_5_vector_got_scalar() {
    let client = make_client();
    let request = make_request(
        "mock-err-5",
        "mock-err-5",
        Input::Object(indexmap::indexmap! {
            "items".into() => Input::Array(vec![
                Input::String("A".into()),
                Input::String("B".into()),
            ]),
        }),
        42,
    );
    let result = run_execution_allow_error(&client, request).await;
    assert!(result.error.is_some(), "expected error on response");
    assert!(
        matches!(result.output, objectiveai::functions::expression::TaskOutputOwned::Err(_)),
        "expected Err output, got: {:?}",
        result.output,
    );
}

/// 4.6: Output returns nested list (Vectors variant).
#[tokio::test]
async fn test_error_4_6_output_vectors_variant() {
    let client = make_client();
    let request = make_request(
        "mock-err-6",
        "mock-err-6",
        Input::Object(indexmap::indexmap! {
            "text".into() => Input::String("test".into()),
        }),
        42,
    );
    let result = run_execution_allow_error(&client, request).await;
    assert!(result.error.is_some(), "expected error on response");
    assert!(
        matches!(result.output, objectiveai::functions::expression::TaskOutputOwned::Err(_)),
        "expected Err output, got: {:?}",
        result.output,
    );
}

/// 4.7: Output expression returns None (Err value).
#[tokio::test]
async fn test_error_4_7_output_returns_none() {
    let client = make_client();
    let request = make_request(
        "mock-err-7",
        "mock-err-7",
        Input::Object(indexmap::indexmap! {
            "text".into() => Input::String("test".into()),
        }),
        42,
    );
    let result = run_execution_allow_error(&client, request).await;
    assert!(result.error.is_some(), "expected error on response");
    assert!(
        matches!(result.output, objectiveai::functions::expression::TaskOutputOwned::Err(_)),
        "expected Err output, got: {:?}",
        result.output,
    );
}

// ---------------------------------------------------------------------------
// 6. Reasoning Errors
// ---------------------------------------------------------------------------

/// 6.1: Reasoning agent error — mock agent with error=true.
#[tokio::test]
async fn test_error_6_1_reasoning_agent_error() {
    let client = make_client();
    let request = make_request_with_body(
        "mock-1",
        "mock-1",
        FunctionRemoteProfileRemoteRequestBody {
            retry_token: None,
            from_cache: None,
            reasoning: Some(objectiveai::functions::executions::request::Reasoning {
                agent: objectiveai::agent::completions::request::Agent::Provided(
                    objectiveai::agent::AgentBase::Mock(objectiveai::agent::mock::AgentBase {
                        upstream: objectiveai::agent::mock::Upstream::Mock,
                        output_mode: objectiveai::agent::mock::OutputMode::Instruction,
                        top_logprobs: None,
                        error: Some(true),
                        invention: None,
                    }),
                ),
                agents: None,
            }),
            strategy: None,
            input: Input::Object(indexmap::indexmap! {
                "text".into() => Input::String("test".into()),
            }),
            provider: None,
            seed: Some(42),
            stream: None,
            mcp_server_authorization: None,
        },
    );
    // The stream succeeds but the reasoning chunk will have an error.
    let result = run_execution_allow_error(&client, request).await;
    // The execution itself should succeed (output is valid).
    // The reasoning should have an error.
    assert!(
        result.reasoning.as_ref().is_some_and(|r| r.error.is_some()),
        "expected reasoning error, got: {:?}",
        result.reasoning,
    );
}
