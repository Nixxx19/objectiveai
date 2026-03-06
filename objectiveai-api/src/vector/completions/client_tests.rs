use std::sync::Arc;
use std::time::Duration;

use futures::StreamExt;
use rust_decimal::Decimal;

use objectiveai::agent::completions::message::{Message, RichContent, UserMessage};
use objectiveai::agent::mock::{AgentBase as MockAgentBase, OutputMode as MockOutputMode, Upstream as MockUpstream};
use objectiveai::vector::completions::response::unary::VectorCompletion;

use crate::agent::completions::UnimplementedUpstreamClient;
use crate::ctx;

// ---------------------------------------------------------------------------
// Stubs — never actually called since we always provide inline mock agents.
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
impl super::completion_votes_fetcher::Fetcher<ctx::DefaultContextExt>
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
impl super::cache_vote_fetcher::Fetcher<ctx::DefaultContextExt>
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
impl super::usage_handler::UsageHandler<ctx::DefaultContextExt>
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

// ---------------------------------------------------------------------------
// Snapshot helpers
// ---------------------------------------------------------------------------

fn aggregate(
    chunks: Vec<objectiveai::vector::completions::response::streaming::VectorCompletionChunk>,
) -> VectorCompletion {
    let mut agg: Option<
        objectiveai::vector::completions::response::streaming::VectorCompletionChunk,
    > = None;
    for chunk in &chunks {
        match &mut agg {
            Some(a) => a.push(chunk),
            None => agg = Some(chunk.clone()),
        }
    }
    agg.expect("stream should have at least one chunk").into()
}

fn normalize(mut vc: VectorCompletion) -> VectorCompletion {
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
    vc
}

fn assert_snapshot(json: &str, path: &str, expected: &str) {
    if std::env::var("UPDATE_SNAPSHOTS").as_deref() == Ok("1") {
        std::fs::write(path, json).unwrap();
        eprintln!("Updated snapshot: {path}");
    } else {
        assert_eq!(json, expected.trim_end());
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Single mock agent, 2 responses, instruction mode, seed 42.
#[tokio::test]
async fn test_single_agent_2_responses_instruction_seed_42() {
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
        mock: Arc::new(crate::agent::completions::mock::client::Client {
            delay: Duration::ZERO,
            seed: Some(42),
            max_tool_calls: Some(0),
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
    });
    let client = Arc::new(super::Client {
        agent_client,
        ensemble_fetcher: Arc::new(crate::ensemble::fetcher::CachingFetcher::new(
            Arc::new(StubEnsembleFetcher),
        )),
        completion_votes_fetcher: Arc::new(StubCompletionVotesFetcher),
        cache_vote_fetcher: Arc::new(StubCacheVoteFetcher),
        usage_handler: Arc::new(StubVectorUsageHandler),
    });
    let request = Arc::new(objectiveai::vector::completions::request::VectorCompletionCreateParams {
        retry: None,
        from_cache: None,
        from_rng: None,
        messages: vec![Message::User(UserMessage {
            content: RichContent::Text("Which is better?".to_string()),
            name: None,
        })],
        provider: None,
        ensemble: objectiveai::vector::completions::request::Ensemble::Provided(
            objectiveai::ensemble::EnsembleBase {
                agents: vec![objectiveai::agent::AgentBaseWithFallbacksAndCount {
                    count: 1,
                    inner: objectiveai::agent::AgentBase::Mock(MockAgentBase {
                            upstream: MockUpstream::Mock,
                            output_mode: MockOutputMode::Instruction,
                            error: None,
                        }),
                    fallbacks: None,
                }],
            },
        ),
        profile: objectiveai::vector::completions::request::Profile::Weights(vec![
            Decimal::ONE,
        ]),
        seed: Some(42),
        stream: None,
        responses: vec![
            RichContent::Text("Response A".to_string()),
            RichContent::Text("Response B".to_string()),
        ],
        mcp_server_authorization: None,
        backoff_max_elapsed_time: None,
        first_chunk_timeout: None,
        other_chunk_timeout: None,
    });

    let stream = client
        .create_streaming(
            ctx::Context::new(Arc::new(ctx::DefaultContextExt), Decimal::ONE),
            request,
        )
        .await
        .expect("create_streaming should succeed");
    let chunks: Vec<_> = Box::pin(stream).collect().await;
    assert!(!chunks.is_empty(), "should have at least one chunk");
    let result = normalize(aggregate(chunks));

    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/vector/completions/client_tests/single_agent_2_responses_instruction_seed_42.json"),
        include_str!("../../../assets/vector/completions/client_tests/single_agent_2_responses_instruction_seed_42.json"),
    );
}

/// Single mock agent, 3 responses, instruction mode, seed 42.
#[tokio::test]
async fn test_single_agent_3_responses_instruction_seed_42() {
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
        mock: Arc::new(crate::agent::completions::mock::client::Client {
            delay: Duration::ZERO,
            seed: Some(42),
            max_tool_calls: Some(0),
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
    });
    let client = Arc::new(super::Client {
        agent_client,
        ensemble_fetcher: Arc::new(crate::ensemble::fetcher::CachingFetcher::new(
            Arc::new(StubEnsembleFetcher),
        )),
        completion_votes_fetcher: Arc::new(StubCompletionVotesFetcher),
        cache_vote_fetcher: Arc::new(StubCacheVoteFetcher),
        usage_handler: Arc::new(StubVectorUsageHandler),
    });
    let request = Arc::new(objectiveai::vector::completions::request::VectorCompletionCreateParams {
        retry: None,
        from_cache: None,
        from_rng: None,
        messages: vec![Message::User(UserMessage {
            content: RichContent::Text("Which is best?".to_string()),
            name: None,
        })],
        provider: None,
        ensemble: objectiveai::vector::completions::request::Ensemble::Provided(
            objectiveai::ensemble::EnsembleBase {
                agents: vec![objectiveai::agent::AgentBaseWithFallbacksAndCount {
                    count: 1,
                    inner: objectiveai::agent::AgentBase::Mock(MockAgentBase {
                            upstream: MockUpstream::Mock,
                            output_mode: MockOutputMode::Instruction,
                            error: None,
                        }),
                    fallbacks: None,
                }],
            },
        ),
        profile: objectiveai::vector::completions::request::Profile::Weights(vec![
            Decimal::ONE,
        ]),
        seed: Some(42),
        stream: None,
        responses: vec![
            RichContent::Text("Alpha".to_string()),
            RichContent::Text("Beta".to_string()),
            RichContent::Text("Gamma".to_string()),
        ],
        mcp_server_authorization: None,
        backoff_max_elapsed_time: None,
        first_chunk_timeout: None,
        other_chunk_timeout: None,
    });

    let stream = client
        .create_streaming(
            ctx::Context::new(Arc::new(ctx::DefaultContextExt), Decimal::ONE),
            request,
        )
        .await
        .expect("create_streaming should succeed");
    let chunks: Vec<_> = Box::pin(stream).collect().await;
    assert!(!chunks.is_empty(), "should have at least one chunk");
    let result = normalize(aggregate(chunks));

    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/vector/completions/client_tests/single_agent_3_responses_instruction_seed_42.json"),
        include_str!("../../../assets/vector/completions/client_tests/single_agent_3_responses_instruction_seed_42.json"),
    );
}

/// Two mock agents with equal weights, seed 42.
#[tokio::test]
async fn test_two_agents_equal_weights_seed_42() {
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
        mock: Arc::new(crate::agent::completions::mock::client::Client {
            delay: Duration::ZERO,
            seed: Some(42),
            max_tool_calls: Some(0),
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
    });
    let client = Arc::new(super::Client {
        agent_client,
        ensemble_fetcher: Arc::new(crate::ensemble::fetcher::CachingFetcher::new(
            Arc::new(StubEnsembleFetcher),
        )),
        completion_votes_fetcher: Arc::new(StubCompletionVotesFetcher),
        cache_vote_fetcher: Arc::new(StubCacheVoteFetcher),
        usage_handler: Arc::new(StubVectorUsageHandler),
    });
    let request = Arc::new(objectiveai::vector::completions::request::VectorCompletionCreateParams {
        retry: None,
        from_cache: None,
        from_rng: None,
        messages: vec![Message::User(UserMessage {
            content: RichContent::Text("Pick one".to_string()),
            name: None,
        })],
        provider: None,
        ensemble: objectiveai::vector::completions::request::Ensemble::Provided(
            objectiveai::ensemble::EnsembleBase {
                agents: vec![objectiveai::agent::AgentBaseWithFallbacksAndCount {
                    count: 2,
                    inner: objectiveai::agent::AgentBase::Mock(MockAgentBase {
                            upstream: MockUpstream::Mock,
                            output_mode: MockOutputMode::Instruction,
                            error: None,
                        }),
                    fallbacks: None,
                }],
            },
        ),
        profile: objectiveai::vector::completions::request::Profile::Weights(vec![
            Decimal::ONE,
        ]),
        seed: Some(42),
        stream: None,
        responses: vec![
            RichContent::Text("Option 1".to_string()),
            RichContent::Text("Option 2".to_string()),
        ],
        mcp_server_authorization: None,
        backoff_max_elapsed_time: None,
        first_chunk_timeout: None,
        other_chunk_timeout: None,
    });

    let stream = client
        .create_streaming(
            ctx::Context::new(Arc::new(ctx::DefaultContextExt), Decimal::ONE),
            request,
        )
        .await
        .expect("create_streaming should succeed");
    let chunks: Vec<_> = Box::pin(stream).collect().await;
    assert!(!chunks.is_empty(), "should have at least one chunk");
    let result = normalize(aggregate(chunks));

    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/vector/completions/client_tests/two_agents_equal_weights_seed_42.json"),
        include_str!("../../../assets/vector/completions/client_tests/two_agents_equal_weights_seed_42.json"),
    );
}

/// Two different mock agent definitions with unequal weights (0.8 / 0.2), seed 42.
#[tokio::test]
async fn test_two_agents_unequal_weights_seed_42() {
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
        mock: Arc::new(crate::agent::completions::mock::client::Client {
            delay: Duration::ZERO,
            seed: Some(42),
            max_tool_calls: Some(0),
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
    });
    let client = Arc::new(super::Client {
        agent_client,
        ensemble_fetcher: Arc::new(crate::ensemble::fetcher::CachingFetcher::new(
            Arc::new(StubEnsembleFetcher),
        )),
        completion_votes_fetcher: Arc::new(StubCompletionVotesFetcher),
        cache_vote_fetcher: Arc::new(StubCacheVoteFetcher),
        usage_handler: Arc::new(StubVectorUsageHandler),
    });
    let request = Arc::new(objectiveai::vector::completions::request::VectorCompletionCreateParams {
        retry: None,
        from_cache: None,
        from_rng: None,
        messages: vec![Message::User(UserMessage {
            content: RichContent::Text("Pick one".to_string()),
            name: None,
        })],
        provider: None,
        ensemble: objectiveai::vector::completions::request::Ensemble::Provided(
            objectiveai::ensemble::EnsembleBase {
                agents: vec![
                    objectiveai::agent::AgentBaseWithFallbacksAndCount {
                        count: 1,
                        inner: objectiveai::agent::AgentBase::Mock(MockAgentBase {
                            upstream: MockUpstream::Mock,
                            output_mode: MockOutputMode::Instruction,
                            error: None,
                        }),
                        fallbacks: None,
                    },
                    objectiveai::agent::AgentBaseWithFallbacksAndCount {
                        count: 1,
                        inner: objectiveai::agent::AgentBase::Mock(MockAgentBase {
                            upstream: MockUpstream::Mock,
                            output_mode: MockOutputMode::Instruction,
                            error: None,
                        }),
                        fallbacks: None,
                    },
                ],
            },
        ),
        profile: objectiveai::vector::completions::request::Profile::Weights(vec![
            Decimal::new(8, 1),
            Decimal::new(2, 1),
        ]),
        seed: Some(42),
        stream: None,
        responses: vec![
            RichContent::Text("Option 1".to_string()),
            RichContent::Text("Option 2".to_string()),
        ],
        mcp_server_authorization: None,
        backoff_max_elapsed_time: None,
        first_chunk_timeout: None,
        other_chunk_timeout: None,
    });

    let stream = client
        .create_streaming(
            ctx::Context::new(Arc::new(ctx::DefaultContextExt), Decimal::ONE),
            request,
        )
        .await
        .expect("create_streaming should succeed");
    let chunks: Vec<_> = Box::pin(stream).collect().await;
    assert!(!chunks.is_empty(), "should have at least one chunk");
    let result = normalize(aggregate(chunks));

    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/vector/completions/client_tests/two_agents_unequal_weights_seed_42.json"),
        include_str!("../../../assets/vector/completions/client_tests/two_agents_unequal_weights_seed_42.json"),
    );
}

/// Three agents (via count=3), 4 responses, seed 99.
#[tokio::test]
async fn test_three_agents_4_responses_seed_99() {
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
        mock: Arc::new(crate::agent::completions::mock::client::Client {
            delay: Duration::ZERO,
            seed: Some(99),
            max_tool_calls: Some(0),
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
    });
    let client = Arc::new(super::Client {
        agent_client,
        ensemble_fetcher: Arc::new(crate::ensemble::fetcher::CachingFetcher::new(
            Arc::new(StubEnsembleFetcher),
        )),
        completion_votes_fetcher: Arc::new(StubCompletionVotesFetcher),
        cache_vote_fetcher: Arc::new(StubCacheVoteFetcher),
        usage_handler: Arc::new(StubVectorUsageHandler),
    });
    let request = Arc::new(objectiveai::vector::completions::request::VectorCompletionCreateParams {
        retry: None,
        from_cache: None,
        from_rng: None,
        messages: vec![Message::User(UserMessage {
            content: RichContent::Text("Rank these".to_string()),
            name: None,
        })],
        provider: None,
        ensemble: objectiveai::vector::completions::request::Ensemble::Provided(
            objectiveai::ensemble::EnsembleBase {
                agents: vec![objectiveai::agent::AgentBaseWithFallbacksAndCount {
                    count: 3,
                    inner: objectiveai::agent::AgentBase::Mock(MockAgentBase {
                            upstream: MockUpstream::Mock,
                            output_mode: MockOutputMode::Instruction,
                            error: None,
                        }),
                    fallbacks: None,
                }],
            },
        ),
        profile: objectiveai::vector::completions::request::Profile::Weights(vec![
            Decimal::ONE,
        ]),
        seed: Some(99),
        stream: None,
        responses: vec![
            RichContent::Text("Red".to_string()),
            RichContent::Text("Green".to_string()),
            RichContent::Text("Blue".to_string()),
            RichContent::Text("Yellow".to_string()),
        ],
        mcp_server_authorization: None,
        backoff_max_elapsed_time: None,
        first_chunk_timeout: None,
        other_chunk_timeout: None,
    });

    let stream = client
        .create_streaming(
            ctx::Context::new(Arc::new(ctx::DefaultContextExt), Decimal::ONE),
            request,
        )
        .await
        .expect("create_streaming should succeed");
    let chunks: Vec<_> = Box::pin(stream).collect().await;
    assert!(!chunks.is_empty(), "should have at least one chunk");
    let result = normalize(aggregate(chunks));

    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/vector/completions/client_tests/three_agents_4_responses_seed_99.json"),
        include_str!("../../../assets/vector/completions/client_tests/three_agents_4_responses_seed_99.json"),
    );
}

/// Invert vote with single agent, seed 42.
#[tokio::test]
async fn test_invert_vote_seed_42() {
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
        mock: Arc::new(crate::agent::completions::mock::client::Client {
            delay: Duration::ZERO,
            seed: Some(42),
            max_tool_calls: Some(0),
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
    });
    let client = Arc::new(super::Client {
        agent_client,
        ensemble_fetcher: Arc::new(crate::ensemble::fetcher::CachingFetcher::new(
            Arc::new(StubEnsembleFetcher),
        )),
        completion_votes_fetcher: Arc::new(StubCompletionVotesFetcher),
        cache_vote_fetcher: Arc::new(StubCacheVoteFetcher),
        usage_handler: Arc::new(StubVectorUsageHandler),
    });
    let request = Arc::new(objectiveai::vector::completions::request::VectorCompletionCreateParams {
        retry: None,
        from_cache: None,
        from_rng: None,
        messages: vec![Message::User(UserMessage {
            content: RichContent::Text("Which is worse?".to_string()),
            name: None,
        })],
        provider: None,
        ensemble: objectiveai::vector::completions::request::Ensemble::Provided(
            objectiveai::ensemble::EnsembleBase {
                agents: vec![objectiveai::agent::AgentBaseWithFallbacksAndCount {
                    count: 1,
                    inner: objectiveai::agent::AgentBase::Mock(MockAgentBase {
                            upstream: MockUpstream::Mock,
                            output_mode: MockOutputMode::Instruction,
                            error: None,
                        }),
                    fallbacks: None,
                }],
            },
        ),
        profile: objectiveai::vector::completions::request::Profile::Entries(vec![
            objectiveai::vector::completions::request::ProfileEntry {
                weight: Decimal::ONE,
                invert: Some(true),
            },
        ]),
        seed: Some(42),
        stream: None,
        responses: vec![
            RichContent::Text("Bad option".to_string()),
            RichContent::Text("Worse option".to_string()),
        ],
        mcp_server_authorization: None,
        backoff_max_elapsed_time: None,
        first_chunk_timeout: None,
        other_chunk_timeout: None,
    });

    let stream = client
        .create_streaming(
            ctx::Context::new(Arc::new(ctx::DefaultContextExt), Decimal::ONE),
            request,
        )
        .await
        .expect("create_streaming should succeed");
    let chunks: Vec<_> = Box::pin(stream).collect().await;
    assert!(!chunks.is_empty(), "should have at least one chunk");
    let result = normalize(aggregate(chunks));

    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/vector/completions/client_tests/invert_vote_seed_42.json"),
        include_str!("../../../assets/vector/completions/client_tests/invert_vote_seed_42.json"),
    );
}

/// Same seed produces same result (deterministic).
#[tokio::test]
async fn test_deterministic_same_seed() {
    let make_client = || {
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
            mock: Arc::new(crate::agent::completions::mock::client::Client {
                delay: Duration::ZERO,
                seed: Some(42),
                max_tool_calls: Some(0),
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
        });
        Arc::new(super::Client {
            agent_client,
            ensemble_fetcher: Arc::new(crate::ensemble::fetcher::CachingFetcher::new(
                Arc::new(StubEnsembleFetcher),
            )),
            completion_votes_fetcher: Arc::new(StubCompletionVotesFetcher),
            cache_vote_fetcher: Arc::new(StubCacheVoteFetcher),
            usage_handler: Arc::new(StubVectorUsageHandler),
        })
    };
    let make_request = || {
        Arc::new(objectiveai::vector::completions::request::VectorCompletionCreateParams {
            retry: None,
            from_cache: None,
            from_rng: None,
            messages: vec![Message::User(UserMessage {
                content: RichContent::Text("Pick one".to_string()),
                name: None,
            })],
            provider: None,
            ensemble: objectiveai::vector::completions::request::Ensemble::Provided(
                objectiveai::ensemble::EnsembleBase {
                    agents: vec![objectiveai::agent::AgentBaseWithFallbacksAndCount {
                        count: 2,
                        inner: objectiveai::agent::AgentBase::Mock(MockAgentBase {
                            upstream: MockUpstream::Mock,
                            output_mode: MockOutputMode::Instruction,
                            error: None,
                        }),
                        fallbacks: None,
                    }],
                },
            ),
            profile: objectiveai::vector::completions::request::Profile::Weights(vec![
                Decimal::ONE,
            ]),
            seed: Some(42),
            stream: None,
            responses: vec![
                RichContent::Text("A".to_string()),
                RichContent::Text("B".to_string()),
                RichContent::Text("C".to_string()),
            ],
            mcp_server_authorization: None,
            backoff_max_elapsed_time: None,
            first_chunk_timeout: None,
            other_chunk_timeout: None,
        })
    };

    let run = |client: Arc<super::Client<_, _, _, _, _, _, _, _, _, _>>, request| async move {
        let stream = client
            .create_streaming(
                ctx::Context::new(Arc::new(ctx::DefaultContextExt), Decimal::ONE),
                request,
            )
            .await
            .expect("should succeed");
        let chunks: Vec<_> = Box::pin(stream).collect().await;
        normalize(aggregate(chunks))
    };

    let result1 = run(make_client(), make_request()).await;
    let result2 = run(make_client(), make_request()).await;

    let json1 = serde_json::to_string_pretty(&result1).unwrap();
    let json2 = serde_json::to_string_pretty(&result2).unwrap();
    assert_eq!(json1, json2, "same seed should produce identical results");
}

/// Different seeds produce different results.
#[tokio::test]
async fn test_different_seeds_differ() {
    let make_client = |seed: u64| {
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
            mock: Arc::new(crate::agent::completions::mock::client::Client {
                delay: Duration::ZERO,
                seed: Some(seed),
                max_tool_calls: Some(0),
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
        });
        Arc::new(super::Client {
            agent_client,
            ensemble_fetcher: Arc::new(crate::ensemble::fetcher::CachingFetcher::new(
                Arc::new(StubEnsembleFetcher),
            )),
            completion_votes_fetcher: Arc::new(StubCompletionVotesFetcher),
            cache_vote_fetcher: Arc::new(StubCacheVoteFetcher),
            usage_handler: Arc::new(StubVectorUsageHandler),
        })
    };
    let make_request = |seed: i64| {
        Arc::new(objectiveai::vector::completions::request::VectorCompletionCreateParams {
            retry: None,
            from_cache: None,
            from_rng: None,
            messages: vec![Message::User(UserMessage {
                content: RichContent::Text("Pick one".to_string()),
                name: None,
            })],
            provider: None,
            ensemble: objectiveai::vector::completions::request::Ensemble::Provided(
                objectiveai::ensemble::EnsembleBase {
                    agents: vec![objectiveai::agent::AgentBaseWithFallbacksAndCount {
                        count: 1,
                        inner: objectiveai::agent::AgentBase::Mock(MockAgentBase {
                            upstream: MockUpstream::Mock,
                            output_mode: MockOutputMode::Instruction,
                            error: None,
                        }),
                        fallbacks: None,
                    }],
                },
            ),
            profile: objectiveai::vector::completions::request::Profile::Weights(vec![
                Decimal::ONE,
            ]),
            seed: Some(seed),
            stream: None,
            responses: vec![
                RichContent::Text("A".to_string()),
                RichContent::Text("B".to_string()),
            ],
            mcp_server_authorization: None,
            backoff_max_elapsed_time: None,
            first_chunk_timeout: None,
            other_chunk_timeout: None,
        })
    };

    let run = |client: Arc<super::Client<_, _, _, _, _, _, _, _, _, _>>, request| async move {
        let stream = client
            .create_streaming(
                ctx::Context::new(Arc::new(ctx::DefaultContextExt), Decimal::ONE),
                request,
            )
            .await
            .expect("should succeed");
        let chunks: Vec<_> = Box::pin(stream).collect().await;
        normalize(aggregate(chunks))
    };

    let result1 = run(make_client(42), make_request(42)).await;
    let result2 = run(make_client(99), make_request(99)).await;

    let json1 = serde_json::to_string_pretty(&result1).unwrap();
    let json2 = serde_json::to_string_pretty(&result2).unwrap();
    assert_ne!(json1, json2, "different seeds should produce different results");
}

/// Many responses (25) to test deep prefix tree, seed 42.
#[tokio::test]
async fn test_many_responses_deep_prefix_tree_seed_42() {
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
        mock: Arc::new(crate::agent::completions::mock::client::Client {
            delay: Duration::ZERO,
            seed: Some(42),
            max_tool_calls: Some(0),
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
    });
    let client = Arc::new(super::Client {
        agent_client,
        ensemble_fetcher: Arc::new(crate::ensemble::fetcher::CachingFetcher::new(
            Arc::new(StubEnsembleFetcher),
        )),
        completion_votes_fetcher: Arc::new(StubCompletionVotesFetcher),
        cache_vote_fetcher: Arc::new(StubCacheVoteFetcher),
        usage_handler: Arc::new(StubVectorUsageHandler),
    });
    let responses: Vec<RichContent> = (0..25)
        .map(|i| RichContent::Text(format!("Response {}", i)))
        .collect();
    let request = Arc::new(objectiveai::vector::completions::request::VectorCompletionCreateParams {
        retry: None,
        from_cache: None,
        from_rng: None,
        messages: vec![Message::User(UserMessage {
            content: RichContent::Text("Pick the best".to_string()),
            name: None,
        })],
        provider: None,
        ensemble: objectiveai::vector::completions::request::Ensemble::Provided(
            objectiveai::ensemble::EnsembleBase {
                agents: vec![objectiveai::agent::AgentBaseWithFallbacksAndCount {
                    count: 1,
                    inner: objectiveai::agent::AgentBase::Mock(MockAgentBase {
                            upstream: MockUpstream::Mock,
                            output_mode: MockOutputMode::Instruction,
                            error: None,
                        }),
                    fallbacks: None,
                }],
            },
        ),
        profile: objectiveai::vector::completions::request::Profile::Weights(vec![
            Decimal::ONE,
        ]),
        seed: Some(42),
        stream: None,
        responses,
        mcp_server_authorization: None,
        backoff_max_elapsed_time: None,
        first_chunk_timeout: None,
        other_chunk_timeout: None,
    });

    let stream = client
        .create_streaming(
            ctx::Context::new(Arc::new(ctx::DefaultContextExt), Decimal::ONE),
            request,
        )
        .await
        .expect("create_streaming should succeed");
    let chunks: Vec<_> = Box::pin(stream).collect().await;
    assert!(!chunks.is_empty(), "should have at least one chunk");
    let result = normalize(aggregate(chunks));

    let json = serde_json::to_string_pretty(&result).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/vector/completions/client_tests/many_responses_deep_prefix_tree_seed_42.json"),
        include_str!("../../../assets/vector/completions/client_tests/many_responses_deep_prefix_tree_seed_42.json"),
    );
}
