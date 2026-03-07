//! Tests for function execution client.

use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use rust_decimal::Decimal;

use objectiveai::functions::executions::request::{
    FunctionRemoteProfileRemoteRequestBody, FunctionRemoteProfileRemoteRequestPath, Request,
};
use objectiveai::functions::executions::response::streaming::FunctionExecutionChunk;
use objectiveai::functions::executions::response::unary::FunctionExecution;
use objectiveai::functions::expression::Input;
use objectiveai::functions::Remote;

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

async fn run_execution(client: &Arc<TestClient>, request: Arc<Request>) -> FunctionExecution {
    let ctx = ctx::Context::new(Arc::new(ctx::DefaultContextExt), Decimal::ONE);
    let stream = client
        .clone()
        .create_streaming(ctx, request)
        .await
        .expect("create_streaming should succeed");
    let chunks: Vec<_> = Box::pin(stream).collect().await;
    assert!(!chunks.is_empty(), "should have at least one chunk");
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
