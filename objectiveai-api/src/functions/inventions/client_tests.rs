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

fn params(name: &str, depth: u64, min_b: u64, max_b: u64, min_l: u64, max_l: u64) -> Params {
    Params {
        depth,
        min_branch_width: min_b,
        max_branch_width: max_b,
        min_leaf_width: min_l,
        max_leaf_width: max_l,
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
// Test macro
// ---------------------------------------------------------------------------

macro_rules! invention_test {
    (
        $test_name:ident,
        $variant:ident, $state_ty:ident,
        $name:expr, $depth:expr,
        $min_b:expr, $max_b:expr, $min_l:expr, $max_l:expr,
        $seed:expr,
        $snapshot:expr
    ) => {
        #[tokio::test]
        async fn $test_name() {
            let client = make_client();
            let request = make_request(
                ParamsState::$variant($state_ty {
                    params: params($name, $depth, $min_b, $max_b, $min_l, $max_l),
                    essay: None,
                    input_schema: None,
                    essay_tasks: None,
                    tasks: None,
                    description: None,
                    readme: None,
                }),
                $seed,
            );
            let result = normalize(run_invention(&client, request).await);
            let json = serde_json::to_string_pretty(&result).unwrap();
            assert_snapshot(
                &json,
                concat!(env!("CARGO_MANIFEST_DIR"), "/assets/functions/inventions/client_tests/", $snapshot),
                include_str!(concat!("../../../assets/functions/inventions/client_tests/", $snapshot)),
            );
        }
    };
}

// ---------------------------------------------------------------------------
// Scalar Leaf tests (depth=0)
// ---------------------------------------------------------------------------

// Default widths (3-5), baseline
invention_test!(test_scalar_leaf_s42,
    AlphaScalarLeaf, AlphaScalarLeafState,
    "sl-default", 0, 3, 5, 3, 5, 42,
    "scalar_leaf_s42.json");

// Minimum width: exactly 1 task
invention_test!(test_scalar_leaf_s7,
    AlphaScalarLeaf, AlphaScalarLeafState,
    "sl-min-1", 0, 1, 1, 1, 1, 7,
    "scalar_leaf_s7.json");

// Narrow range: 2-3
invention_test!(test_scalar_leaf_s1337,
    AlphaScalarLeaf, AlphaScalarLeafState,
    "sl-narrow", 0, 2, 3, 2, 3, 1337,
    "scalar_leaf_s1337.json");

// Large width: 10 tasks
invention_test!(test_scalar_leaf_s999,
    AlphaScalarLeaf, AlphaScalarLeafState,
    "sl-wide-10", 0, 10, 10, 10, 10, 999,
    "scalar_leaf_s999.json");

// Asymmetric: narrow branch, wide leaf
invention_test!(test_scalar_leaf_s314,
    AlphaScalarLeaf, AlphaScalarLeafState,
    "sl-asym", 0, 1, 2, 7, 10, 314,
    "scalar_leaf_s314.json");

// Wide range
invention_test!(test_scalar_leaf_s8675309,
    AlphaScalarLeaf, AlphaScalarLeafState,
    "sl-range", 0, 1, 10, 1, 8, 8675309,
    "scalar_leaf_s8675309.json");

// ---------------------------------------------------------------------------
// Scalar Branch tests (depth>=1)
// ---------------------------------------------------------------------------

// Default widths, depth 1
invention_test!(test_scalar_branch_s42,
    AlphaScalarBranch, AlphaScalarBranchState,
    "sb-default", 1, 3, 5, 3, 5, 42,
    "scalar_branch_s42.json");

// Minimum width, depth 1
invention_test!(test_scalar_branch_s13,
    AlphaScalarBranch, AlphaScalarBranchState,
    "sb-min-1", 1, 1, 1, 1, 1, 13,
    "scalar_branch_s13.json");

// Narrow: exactly 2 tasks
invention_test!(test_scalar_branch_s2718,
    AlphaScalarBranch, AlphaScalarBranchState,
    "sb-narrow", 1, 2, 2, 2, 2, 2718,
    "scalar_branch_s2718.json");

// Large width, depth 2
invention_test!(test_scalar_branch_s77777,
    AlphaScalarBranch, AlphaScalarBranchState,
    "sb-wide-d2", 2, 10, 10, 10, 10, 77777,
    "scalar_branch_s77777.json");

// Asymmetric: wide branch, narrow leaf
invention_test!(test_scalar_branch_s555,
    AlphaScalarBranch, AlphaScalarBranchState,
    "sb-asym", 1, 8, 10, 1, 2, 555,
    "scalar_branch_s555.json");

// Deep depth 3, narrow
invention_test!(test_scalar_branch_s161803,
    AlphaScalarBranch, AlphaScalarBranchState,
    "sb-deep", 3, 2, 3, 2, 3, 161803,
    "scalar_branch_s161803.json");

// ---------------------------------------------------------------------------
// Vector Leaf tests (depth=0)
// ---------------------------------------------------------------------------

// Default widths
invention_test!(test_vector_leaf_s42,
    AlphaVectorLeaf, AlphaVectorLeafState,
    "vl-default", 0, 3, 5, 3, 5, 42,
    "vector_leaf_s42.json");

// Minimum width
invention_test!(test_vector_leaf_s23,
    AlphaVectorLeaf, AlphaVectorLeafState,
    "vl-min-1", 0, 1, 1, 1, 1, 23,
    "vector_leaf_s23.json");

// Narrow: exactly 2
invention_test!(test_vector_leaf_s404,
    AlphaVectorLeaf, AlphaVectorLeafState,
    "vl-narrow", 0, 2, 2, 2, 2, 404,
    "vector_leaf_s404.json");

// Large width
invention_test!(test_vector_leaf_s31415,
    AlphaVectorLeaf, AlphaVectorLeafState,
    "vl-wide-10", 0, 10, 10, 10, 10, 31415,
    "vector_leaf_s31415.json");

// Asymmetric
invention_test!(test_vector_leaf_s65536,
    AlphaVectorLeaf, AlphaVectorLeafState,
    "vl-asym", 0, 2, 3, 6, 10, 65536,
    "vector_leaf_s65536.json");

// Wide range
invention_test!(test_vector_leaf_s271828,
    AlphaVectorLeaf, AlphaVectorLeafState,
    "vl-range", 0, 1, 10, 1, 10, 271828,
    "vector_leaf_s271828.json");

// ---------------------------------------------------------------------------
// Vector Branch tests (depth>=1)
// ---------------------------------------------------------------------------

// Default widths, depth 1
invention_test!(test_vector_branch_s42,
    AlphaVectorBranch, AlphaVectorBranchState,
    "vb-default", 1, 3, 5, 3, 5, 42,
    "vector_branch_s42.json");

// Minimum width
invention_test!(test_vector_branch_s71,
    AlphaVectorBranch, AlphaVectorBranchState,
    "vb-min-1", 1, 1, 1, 1, 1, 71,
    "vector_branch_s71.json");

// Narrow: exactly 2
invention_test!(test_vector_branch_s12345,
    AlphaVectorBranch, AlphaVectorBranchState,
    "vb-narrow", 1, 2, 2, 2, 2, 12345,
    "vector_branch_s12345.json");

// Large width, depth 2
invention_test!(test_vector_branch_s90210,
    AlphaVectorBranch, AlphaVectorBranchState,
    "vb-wide-d2", 2, 10, 10, 10, 10, 90210,
    "vector_branch_s90210.json");

// Asymmetric: narrow branch, wide leaf
invention_test!(test_vector_branch_s1984,
    AlphaVectorBranch, AlphaVectorBranchState,
    "vb-asym", 1, 1, 2, 8, 10, 1984,
    "vector_branch_s1984.json");

// Deep depth 3
invention_test!(test_vector_branch_s2025,
    AlphaVectorBranch, AlphaVectorBranchState,
    "vb-deep", 3, 2, 4, 2, 4, 2025,
    "vector_branch_s2025.json");

// ---------------------------------------------------------------------------
// Validation error tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_zero_leaf_width_rejected() {
    let client = make_client();
    let ctx = ctx::Context::new(Arc::new(ctx::DefaultContextExt), Decimal::ONE);
    let request = make_request(
        ParamsState::AlphaScalarLeaf(AlphaScalarLeafState {
            params: params("bad-zero", 0, 3, 5, 0, 0),
            essay: None, input_schema: None, essay_tasks: None,
            tasks: None, description: None, readme: None,
        }),
        1,
    );
    let result = client.create_streaming(ctx, request).await;
    assert!(result.is_err(), "zero leaf width should be rejected");
}

#[tokio::test]
async fn test_zero_branch_width_rejected() {
    let client = make_client();
    let ctx = ctx::Context::new(Arc::new(ctx::DefaultContextExt), Decimal::ONE);
    let request = make_request(
        ParamsState::AlphaScalarBranch(AlphaScalarBranchState {
            params: params("bad-zero-branch", 1, 0, 0, 3, 5),
            essay: None, input_schema: None, essay_tasks: None,
            tasks: None, description: None, readme: None,
        }),
        2,
    );
    let result = client.create_streaming(ctx, request).await;
    assert!(result.is_err(), "zero branch width should be rejected");
}

#[tokio::test]
async fn test_min_greater_than_max_rejected() {
    let client = make_client();
    let ctx = ctx::Context::new(Arc::new(ctx::DefaultContextExt), Decimal::ONE);
    let request = make_request(
        ParamsState::AlphaVectorLeaf(AlphaVectorLeafState {
            params: params("bad-inverted", 0, 5, 3, 5, 3),
            essay: None, input_schema: None, essay_tasks: None,
            tasks: None, description: None, readme: None,
        }),
        3,
    );
    let result = client.create_streaming(ctx, request).await;
    assert!(result.is_err(), "min > max should be rejected");
}
