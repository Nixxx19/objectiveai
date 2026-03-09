//! Tests for recursive function invention client.

use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use rust_decimal::Decimal;

use objectiveai::agent::completions::request::Agent as AgentParam;
use objectiveai::agent::AgentBase;
use objectiveai::functions::inventions::recursive::request::FunctionInventionRecursiveCreateParams;
use objectiveai::functions::inventions::recursive::response::streaming::FunctionInventionRecursiveChunk;
use objectiveai::functions::inventions::recursive::response::unary::FunctionInventionRecursive;
use objectiveai::functions::inventions::state::{Params, ParamsState};
use objectiveai::functions::inventions::state::{
    AlphaScalarLeafState, AlphaScalarBranchState,
    AlphaVectorLeafState, AlphaVectorBranchState,
    AlphaScalarState, AlphaVectorState,
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
impl crate::functions::inventions::usage_handler::UsageHandler<ctx::DefaultContextExt>
    for StubInventionUsageHandler
{
    async fn handle_usage(
        &self,
        _ctx: ctx::Context<ctx::DefaultContextExt>,
        _request: Arc<objectiveai::functions::inventions::request::FunctionInventionCreateParams>,
        _response: objectiveai::functions::inventions::response::unary::FunctionInvention,
    ) {
    }
}

struct StubRecursiveUsageHandler;

#[async_trait::async_trait]
impl super::usage_handler::UsageHandler<ctx::DefaultContextExt>
    for StubRecursiveUsageHandler
{
    async fn handle_usage(
        &self,
        _ctx: ctx::Context<ctx::DefaultContextExt>,
        _request: Arc<FunctionInventionRecursiveCreateParams>,
        _response: FunctionInventionRecursive,
    ) {
    }
}

// ---------------------------------------------------------------------------
// Client construction
// ---------------------------------------------------------------------------

type TestInventionClient = crate::functions::inventions::Client<
    ctx::DefaultContextExt,
    UnimplementedUpstreamClient,
    UnimplementedUpstreamClient,
    crate::agent::completions::mock::Client,
    StubAgentFetcher,
    StubAgentUsageHandler,
    StubInventionUsageHandler,
>;

type TestClient = super::Client<
    ctx::DefaultContextExt,
    UnimplementedUpstreamClient,
    UnimplementedUpstreamClient,
    crate::agent::completions::mock::Client,
    StubAgentFetcher,
    StubAgentUsageHandler,
    StubInventionUsageHandler,
    StubRecursiveUsageHandler,
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
    let github_client = Arc::new(crate::github::Client::new(
        reqwest::Client::new(),
        None,
        None,
        None,
        None,
        backoff::ExponentialBackoff::default(),
    ));
    let filesystem_client = Arc::new(crate::filesystem::Client::new(
        std::path::PathBuf::from("/tmp/objectiveai-test-recursive"),
        "ObjectiveAI".to_string(),
        "noreply@objective-ai.io".to_string(),
    ));
    let invention_client = Arc::new(crate::functions::inventions::Client::new(
        agent_client,
        github_client,
        filesystem_client,
        Arc::new(StubInventionUsageHandler),
        true,
    ));
    Arc::new(super::Client::new(
        invention_client,
        Arc::new(StubRecursiveUsageHandler),
    ))
}

fn make_request(state: ParamsState, seed: i64) -> Arc<FunctionInventionRecursiveCreateParams> {
    Arc::new(FunctionInventionRecursiveCreateParams {
        remote: objectiveai::functions::Remote::Mock,
        name: "test/recursive".to_string(),
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
        seed: Some(seed),
        stream: Some(true),
        max_step_retries: Some(1),
        mcp_server_authorization: None,
    })
}

fn params(name: &str, depth: u64, min_b: u64, max_b: u64, min_l: u64, max_l: u64) -> Params {
    Params {
        depth,
        min_branch_width: min_b,
        max_branch_width: max_b,
        min_leaf_width: min_l,
        max_leaf_width: max_l,
        name: name.to_string(),
        spec: "Test function spec for mock recursive invention.".to_string(),
    }
}

// ---------------------------------------------------------------------------
// Streaming + aggregation helpers
// ---------------------------------------------------------------------------

fn assert_chunk_invariants(chunks: &[FunctionInventionRecursiveChunk]) {
    assert!(!chunks.is_empty(), "stream must not be empty");
    for (i, chunk) in chunks.iter().enumerate() {
        if i < chunks.len() - 1 {
            assert_eq!(
                chunk.inventions.len(), 1,
                "chunk {i} (non-final) has {} invention chunks, expected exactly 1",
                chunk.inventions.len(),
            );
            assert!(
                chunk.usage.is_none(),
                "chunk {i} (non-final) has usage, expected None",
            );
        } else {
            assert_eq!(
                chunk.inventions.len(), 0,
                "final chunk {i} has {} invention chunks, expected 0",
                chunk.inventions.len(),
            );
            assert!(
                chunk.usage.is_some(),
                "final chunk {i} has no usage, expected Some",
            );
        }
    }
}

fn aggregate(chunks: Vec<FunctionInventionRecursiveChunk>) -> FunctionInventionRecursive {
    let mut agg: Option<FunctionInventionRecursiveChunk> = None;
    for chunk in &chunks {
        match &mut agg {
            Some(a) => a.push(chunk),
            None => agg = Some(chunk.clone()),
        }
    }
    FunctionInventionRecursive::from(agg.expect("stream should have at least one chunk"))
}

async fn run_recursive_invention(
    client: &Arc<TestClient>,
    request: Arc<FunctionInventionRecursiveCreateParams>,
) -> FunctionInventionRecursive {
    let client = Arc::clone(client);
    let (tx, rx) = std::sync::mpsc::channel();

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
            assert_chunk_invariants(&chunks);
            aggregate(chunks)
        });
        let _ = tx.send(result);
    });

    rx.recv_timeout(Duration::from_secs(120))
        .expect("recursive invention timed out after 120s")
}

// ---------------------------------------------------------------------------
// Snapshot helpers
// ---------------------------------------------------------------------------

fn normalize(mut fi: FunctionInventionRecursive) -> FunctionInventionRecursive {
    fi.id = String::new();
    fi.created = 0;
    for invention in &mut fi.inventions {
        invention.inner.id = String::new();
        invention.inner.created = 0;
        for completion in &mut invention.inner.completions {
            completion.inner.id = String::new();
            completion.inner.created = 0;
            for msg in &mut completion.inner.messages {
                if let objectiveai::agent::completions::response::unary::Message::Assistant(asst) = msg {
                    asst.upstream_id = String::new();
                    asst.created = 0;
                }
            }
        }
    }
    // Sort inventions by state name and renumber indices sequentially.
    // select_all merges concurrent child streams in non-deterministic order,
    // so both the vec order and ChoiceIndexer indices depend on scheduling.
    // The state name is deterministic (derived from parent name + path index).
    fi.inventions.sort_by(|a, b| {
        a.inner.state.name().cmp(b.inner.state.name())
    });
    for (i, inv) in fi.inventions.iter_mut().enumerate() {
        inv.index = i as u64;
    }
    fi
}

fn assert_snapshot(json: &str, path: &str, expected: &str) {
    if std::env::var("UPDATE_FUNCTIONS_INVENTIONS_RECURSIVE_CLIENT_TESTS_SNAPSHOTS").as_deref() == Ok("1") {
        std::fs::write(path, json).unwrap();
        eprintln!("Updated snapshot: {path}");
        let written = std::fs::read_to_string(path).unwrap();
        assert_eq!(json, written.trim_end());
    } else {
        assert_eq!(json, expected.trim_end());
    }
}

// ---------------------------------------------------------------------------
// Test macro — 3 seeds per test (recursive tests are heavier)
// ---------------------------------------------------------------------------

macro_rules! recursive_test_3x {
    (
        $test_name:ident,
        $variant:ident, $state_ty:ident,
        $name:expr, $depth:expr,
        $min_b:expr, $max_b:expr, $min_l:expr, $max_l:expr,
        $base_seed:expr,
        $base:expr
    ) => {
        mod $test_name {
            use super::*;

            fn make_state(seed_offset: i64) -> (ParamsState, i64) {
                (
                    ParamsState::$variant($state_ty {
                        params: params($name, $depth, $min_b, $max_b, $min_l, $max_l),
                        essay: None,
                        input_schema: None,
                        essay_tasks: None,
                        tasks: None,
                        tasks_length: None,
                        description: None,
                        readme: None,
                    }),
                    ($base_seed as i64) + seed_offset,
                )
            }

            fn run_snapshot(offset: i64, path: &str, expected: &str) {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let client = make_client();
                    let (state, seed) = make_state(offset);
                    let request = make_request(state, seed);
                    let result = normalize(run_recursive_invention(&client, request).await);
                    let json = serde_json::to_string_pretty(&result).unwrap();
                    assert_snapshot(&json, path, expected);
                });
            }

            #[test]
            fn seed_0() {
                run_snapshot(
                    0,
                    concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/inventions/recursive_client_tests/", $base, "_0.json"),
                    include_str!(concat!("../../../../assets/functions/inventions/recursive_client_tests/", $base, "_0.json")),
                );
            }

            #[test]
            fn seed_1() {
                run_snapshot(
                    1,
                    concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/inventions/recursive_client_tests/", $base, "_1.json"),
                    include_str!(concat!("../../../../assets/functions/inventions/recursive_client_tests/", $base, "_1.json")),
                );
            }

            #[test]
            fn seed_2() {
                run_snapshot(
                    2,
                    concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/inventions/recursive_client_tests/", $base, "_2.json"),
                    include_str!(concat!("../../../../assets/functions/inventions/recursive_client_tests/", $base, "_2.json")),
                );
            }
        }
    };
}

/// Same as `recursive_test_3x!` but uses AlphaScalar/AlphaVector (unrouted state).
macro_rules! recursive_test_3x_unrouted {
    (
        $test_name:ident,
        $variant:ident, $state_ty:ident,
        $name:expr, $depth:expr,
        $min_b:expr, $max_b:expr, $min_l:expr, $max_l:expr,
        $base_seed:expr,
        $base:expr
    ) => {
        mod $test_name {
            use super::*;

            fn make_state(seed_offset: i64) -> (ParamsState, i64) {
                (
                    ParamsState::$variant($state_ty {
                        params: params($name, $depth, $min_b, $max_b, $min_l, $max_l),
                        input_schema: None,
                    }),
                    ($base_seed as i64) + seed_offset,
                )
            }

            fn run_snapshot(offset: i64, path: &str, expected: &str) {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async {
                    let client = make_client();
                    let (state, seed) = make_state(offset);
                    let request = make_request(state, seed);
                    let result = normalize(run_recursive_invention(&client, request).await);
                    let json = serde_json::to_string_pretty(&result).unwrap();
                    assert_snapshot(&json, path, expected);
                });
            }

            #[test]
            fn seed_0() {
                run_snapshot(
                    0,
                    concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/inventions/recursive_client_tests/", $base, "_0.json"),
                    include_str!(concat!("../../../../assets/functions/inventions/recursive_client_tests/", $base, "_0.json")),
                );
            }

            #[test]
            fn seed_1() {
                run_snapshot(
                    1,
                    concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/inventions/recursive_client_tests/", $base, "_1.json"),
                    include_str!(concat!("../../../../assets/functions/inventions/recursive_client_tests/", $base, "_1.json")),
                );
            }

            #[test]
            fn seed_2() {
                run_snapshot(
                    2,
                    concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/inventions/recursive_client_tests/", $base, "_2.json"),
                    include_str!(concat!("../../../../assets/functions/inventions/recursive_client_tests/", $base, "_2.json")),
                );
            }
        }
    };
}

// ---------------------------------------------------------------------------
// Leaf tests (depth=0) — just 2, since recursive is wasteful at depth 0
// ---------------------------------------------------------------------------

recursive_test_3x!(test_scalar_leaf_d0,
    AlphaScalarLeaf, AlphaScalarLeafState,
    "rsl-baseline", 0, 1, 1, 2, 4, 100,
    "scalar_leaf_d0");

recursive_test_3x!(test_vector_leaf_d0,
    AlphaVectorLeaf, AlphaVectorLeafState,
    "rvl-baseline", 0, 1, 1, 2, 4, 200,
    "vector_leaf_d0");

// ---------------------------------------------------------------------------
// Depth 1 — scalar (diverse widths and configs)
// ---------------------------------------------------------------------------

// Scalar branch, depth 1, minimum: 1 branch task, 1 leaf task
recursive_test_3x!(test_scalar_d1_min,
    AlphaScalarBranch, AlphaScalarBranchState,
    "rsb-d1-min", 1, 1, 1, 1, 1, 1000,
    "scalar_d1_min");

// Scalar branch, depth 1, default widths 3-5
recursive_test_3x!(test_scalar_d1_default,
    AlphaScalarBranch, AlphaScalarBranchState,
    "rsb-d1-default", 1, 3, 5, 3, 5, 1100,
    "scalar_d1_default");

// Scalar branch, depth 1, narrow branch + wide leaf
recursive_test_3x!(test_scalar_d1_narrow_branch_wide_leaf,
    AlphaScalarBranch, AlphaScalarBranchState,
    "rsb-d1-nbwl", 1, 1, 2, 6, 8, 1200,
    "scalar_d1_narrow_branch_wide_leaf");

// Scalar branch, depth 1, wide branch + narrow leaf
recursive_test_3x!(test_scalar_d1_wide_branch_narrow_leaf,
    AlphaScalarBranch, AlphaScalarBranchState,
    "rsb-d1-wbnl", 1, 6, 8, 1, 2, 1300,
    "scalar_d1_wide_branch_narrow_leaf");

// Scalar branch, depth 1, exact 4 tasks each
recursive_test_3x!(test_scalar_d1_exact_4,
    AlphaScalarBranch, AlphaScalarBranchState,
    "rsb-d1-exact4", 1, 4, 4, 4, 4, 1400,
    "scalar_d1_exact_4");

// Scalar, depth 1, unrouted (AlphaScalar routes to branch)
recursive_test_3x_unrouted!(test_scalar_d1_unrouted,
    AlphaScalar, AlphaScalarState,
    "rs-d1-unrouted", 1, 2, 3, 2, 3, 1500,
    "scalar_d1_unrouted");

// ---------------------------------------------------------------------------
// Depth 1 — vector (diverse widths and configs)
// ---------------------------------------------------------------------------

// Vector branch, depth 1, minimum widths
recursive_test_3x!(test_vector_d1_min,
    AlphaVectorBranch, AlphaVectorBranchState,
    "rvb-d1-min", 1, 1, 1, 1, 1, 2000,
    "vector_d1_min");

// Vector branch, depth 1, default widths
recursive_test_3x!(test_vector_d1_default,
    AlphaVectorBranch, AlphaVectorBranchState,
    "rvb-d1-default", 1, 3, 5, 3, 5, 2100,
    "vector_d1_default");

// Vector branch, depth 1, wide branch + narrow leaf
recursive_test_3x!(test_vector_d1_wide_branch,
    AlphaVectorBranch, AlphaVectorBranchState,
    "rvb-d1-wb", 1, 5, 8, 1, 2, 2200,
    "vector_d1_wide_branch");

// Vector branch, depth 1, narrow branch + wide leaf
recursive_test_3x!(test_vector_d1_narrow_branch,
    AlphaVectorBranch, AlphaVectorBranchState,
    "rvb-d1-nb", 1, 1, 2, 5, 8, 2300,
    "vector_d1_narrow_branch");

// Vector branch, depth 1, exact 3
recursive_test_3x!(test_vector_d1_exact_3,
    AlphaVectorBranch, AlphaVectorBranchState,
    "rvb-d1-exact3", 1, 3, 3, 3, 3, 2400,
    "vector_d1_exact_3");

// Vector, depth 1, unrouted (AlphaVector routes to branch)
recursive_test_3x_unrouted!(test_vector_d1_unrouted,
    AlphaVector, AlphaVectorState,
    "rv-d1-unrouted", 1, 2, 4, 2, 4, 2500,
    "vector_d1_unrouted");

// Vector branch, depth 1, asymmetric range 1-10
recursive_test_3x!(test_vector_d1_wide_range,
    AlphaVectorBranch, AlphaVectorBranchState,
    "rvb-d1-range", 1, 1, 10, 1, 10, 2600,
    "vector_d1_wide_range");

// ---------------------------------------------------------------------------
// Depth 2 — scalar and vector
// ---------------------------------------------------------------------------

// Scalar branch, depth 2, narrow (2-3 tasks per level)
recursive_test_3x!(test_scalar_d2_narrow,
    AlphaScalarBranch, AlphaScalarBranchState,
    "rsb-d2-narrow", 2, 2, 3, 2, 3, 3000,
    "scalar_d2_narrow");

// Scalar branch, depth 2, minimum
recursive_test_3x!(test_scalar_d2_min,
    AlphaScalarBranch, AlphaScalarBranchState,
    "rsb-d2-min", 2, 1, 1, 1, 1, 3100,
    "scalar_d2_min");

// Vector branch, depth 2, default widths
recursive_test_3x!(test_vector_d2_default,
    AlphaVectorBranch, AlphaVectorBranchState,
    "rvb-d2-default", 2, 3, 5, 3, 5, 3200,
    "vector_d2_default");

// Vector branch, depth 2, narrow
recursive_test_3x!(test_vector_d2_narrow,
    AlphaVectorBranch, AlphaVectorBranchState,
    "rvb-d2-narrow", 2, 2, 3, 2, 3, 3300,
    "vector_d2_narrow");

// Scalar, depth 2, unrouted
recursive_test_3x_unrouted!(test_scalar_d2_unrouted,
    AlphaScalar, AlphaScalarState,
    "rs-d2-unrouted", 2, 1, 2, 1, 2, 3400,
    "scalar_d2_unrouted");

// ---------------------------------------------------------------------------
// Depth 3 — just one (expensive)
// ---------------------------------------------------------------------------

// Scalar branch, depth 3, minimum widths to keep it tractable
recursive_test_3x!(test_scalar_d3_min,
    AlphaScalarBranch, AlphaScalarBranchState,
    "rsb-d3-min", 3, 1, 1, 1, 1, 4000,
    "scalar_d3_min");
