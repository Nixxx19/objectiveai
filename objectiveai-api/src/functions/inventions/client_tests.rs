//! Tests for function invention client.

use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use rust_decimal::Decimal;

use objectiveai::agent::completions::request::Agent as AgentParam;
use objectiveai::agent::AgentBase;
use objectiveai::functions::inventions::request::FunctionInventionCreateParams;
use objectiveai::functions::inventions::response::streaming::FunctionInventionChunk;
use objectiveai::functions::inventions::response::unary::FunctionInvention;
use objectiveai::functions::inventions::state::{Params, ParamsState};
use objectiveai::functions::inventions::state::{
    AlphaScalarLeafState, AlphaScalarBranchState,
    AlphaVectorLeafState, AlphaVectorBranchState,
};

use crate::agent::completions::UnimplementedUpstreamClient;
use crate::ctx;

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

struct StubInventionUsageHandler;

#[async_trait::async_trait]
impl super::usage_handler::UsageHandler<ctx::DefaultContextExt> for StubInventionUsageHandler {
    async fn handle_usage(
        &self,
        _ctx: ctx::Context<ctx::DefaultContextExt>,
        _request: Arc<FunctionInventionCreateParams>,
        _response: FunctionInvention,
    ) {
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
    StubInventionUsageHandler,
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
    Arc::new(super::Client::new(agent_client, Arc::new(StubInventionUsageHandler)))
}

fn make_request(state: ParamsState, seed: i64) -> Arc<FunctionInventionCreateParams> {
    Arc::new(FunctionInventionCreateParams {
        remote: None,
        name: None,
        github_token: None,
        state,
        provider: None,
        agent: AgentParam::Provided(AgentBase::Mock(
            objectiveai::agent::mock::AgentBase {
                invention: Some(true),
                ..Default::default()
            },
        )),
        agents: None,
        top_logprobs: None,
        seed: Some(seed),
        stream: Some(true),
        max_step_retries: Some(1),
        mcp_server_authorization: None,
    })
}

fn default_params(name: &str, depth: u64) -> Params {
    Params {
        depth,
        min_branch_width: 3,
        max_branch_width: 5,
        min_leaf_width: 3,
        max_leaf_width: 5,
        name: name.to_string(),
        spec: "Test function spec for mock invention.".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Streaming + aggregation helpers
// ---------------------------------------------------------------------------

fn aggregate(chunks: Vec<FunctionInventionChunk>) -> FunctionInvention {
    let mut agg: Option<FunctionInventionChunk> = None;
    for chunk in &chunks {
        match &mut agg {
            Some(a) => a.push(chunk),
            None => agg = Some(chunk.clone()),
        }
    }
    FunctionInvention::from(agg.expect("stream should have at least one chunk"))
}

async fn run_invention(
    client: &Arc<TestClient>,
    request: Arc<FunctionInventionCreateParams>,
) -> FunctionInvention {
    let client = Arc::clone(client);
    let (tx, rx) = std::sync::mpsc::channel();

    // OS thread + its own tokio runtime.  Immune to any busy loop blocking
    // the caller's async runtime — recv_timeout uses OS-level timing and
    // the OS preemptively schedules threads even on a single CPU core.
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let result = rt.block_on(async {
            let ctx = ctx::Context::new(Arc::new(ctx::DefaultContextExt), Decimal::ONE);
            let stream = client
                .clone()
                .create_streaming(ctx, request)
                .await
                .expect("create_streaming should succeed");
            let chunks: Vec<_> = Box::pin(stream).collect::<Vec<_>>().await;
            assert!(!chunks.is_empty(), "stream must not be empty");
            aggregate(chunks)
        });
        let _ = tx.send(result);
    });

    rx.recv_timeout(Duration::from_secs(10))
        .expect("invention timed out after 10s — check debug logs above")
}

// ---------------------------------------------------------------------------
// Snapshot helpers
// ---------------------------------------------------------------------------

fn normalize(mut fi: FunctionInvention) -> FunctionInvention {
    fi.id = String::new();
    fi.created = 0;
    for completion in &mut fi.completions {
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
    fi
}

fn assert_snapshot(json: &str, path: &str, expected: &str) {
    if std::env::var("UPDATE_FUNCTIONS_INVENTIONS_CLIENT_TESTS_SNAPSHOTS").as_deref() == Ok("1") {
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

/// Scalar leaf invention, seed 42.
#[tokio::test]
async fn test_scalar_leaf_seed_42() {
    let client = make_client();
    let request = make_request(
        ParamsState::AlphaScalarLeaf(AlphaScalarLeafState {
            params: default_params("test-scalar-leaf", 0),
            essay: None,
            input_schema: None,
            essay_tasks: None,
            tasks: None,
            description: None,
            readme: None,
        }),
        42,
    );
    let result = normalize(run_invention(&client, request).await);
    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/inventions/client_tests/scalar_leaf_seed_42.json"),
        include_str!("../../../assets/functions/inventions/client_tests/scalar_leaf_seed_42.json"),
    );
}

/// Scalar branch invention, seed 42.
#[tokio::test]
async fn test_scalar_branch_seed_42() {
    let client = make_client();
    let request = make_request(
        ParamsState::AlphaScalarBranch(AlphaScalarBranchState {
            params: default_params("test-scalar-branch", 1),
            essay: None,
            input_schema: None,
            essay_tasks: None,
            tasks: None,
            description: None,
            readme: None,
        }),
        42,
    );
    let result = normalize(run_invention(&client, request).await);
    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/inventions/client_tests/scalar_branch_seed_42.json"),
        include_str!("../../../assets/functions/inventions/client_tests/scalar_branch_seed_42.json"),
    );
}

/// Vector leaf invention, seed 42.
#[tokio::test]
async fn test_vector_leaf_seed_42() {
    let client = make_client();
    let request = make_request(
        ParamsState::AlphaVectorLeaf(AlphaVectorLeafState {
            params: default_params("test-vector-leaf", 0),
            essay: None,
            input_schema: None,
            essay_tasks: None,
            tasks: None,
            description: None,
            readme: None,
        }),
        42,
    );
    let result = normalize(run_invention(&client, request).await);
    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/inventions/client_tests/vector_leaf_seed_42.json"),
        include_str!("../../../assets/functions/inventions/client_tests/vector_leaf_seed_42.json"),
    );
}

/// Vector branch invention, seed 42.
#[tokio::test]
async fn test_vector_branch_seed_42() {
    let client = make_client();
    let request = make_request(
        ParamsState::AlphaVectorBranch(AlphaVectorBranchState {
            params: default_params("test-vector-branch", 1),
            essay: None,
            input_schema: None,
            essay_tasks: None,
            tasks: None,
            description: None,
            readme: None,
        }),
        42,
    );
    let result = normalize(run_invention(&client, request).await);
    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/inventions/client_tests/vector_branch_seed_42.json"),
        include_str!("../../../assets/functions/inventions/client_tests/vector_branch_seed_42.json"),
    );
}

/// Scalar leaf invention, different seed (99).
#[tokio::test]
async fn test_scalar_leaf_seed_99() {
    let client = make_client();
    let request = make_request(
        ParamsState::AlphaScalarLeaf(AlphaScalarLeafState {
            params: default_params("test-scalar-leaf-alt", 0),
            essay: None,
            input_schema: None,
            essay_tasks: None,
            tasks: None,
            description: None,
            readme: None,
        }),
        99,
    );
    let result = normalize(run_invention(&client, request).await);
    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/inventions/client_tests/scalar_leaf_seed_99.json"),
        include_str!("../../../assets/functions/inventions/client_tests/scalar_leaf_seed_99.json"),
    );
}
