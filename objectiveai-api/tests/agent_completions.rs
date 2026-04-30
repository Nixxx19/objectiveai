//! Integration tests for the `/agent/completions` endpoint of the
//! spawned `objectiveai-api` server. Each test POSTs an
//! `AgentCompletionCreateParams` body, streams the SSE response, and
//! snapshots the aggregated `AgentCompletion`.

#![allow(clippy::too_many_arguments)]

use futures::StreamExt;
use objectiveai::agent::completions::request::AgentCompletionCreateParams;
use objectiveai::agent::completions::response::streaming::AgentCompletionChunk;
use objectiveai::agent::completions::response::unary::AgentCompletion;
use objectiveai::agent::mock::AgentBase as MockAgentBase;

mod common;

// ---------------------------------------------------------------------------
// Snapshot helpers
// ---------------------------------------------------------------------------

fn check_created_and_upstream(
    expected_created: &std::cell::Cell<Option<u64>>,
    expected_upstream: &std::cell::Cell<Option<objectiveai::agent::Upstream>>,
    i: usize,
    chunk: &AgentCompletionChunk,
) {
    match expected_created.get() {
        None => expected_created.set(Some(chunk.created)),
        Some(exp) => assert_eq!(
            chunk.created, exp,
            "chunk {i} has created {}, expected {exp}",
            chunk.created
        ),
    }
    match expected_upstream.get() {
        None => expected_upstream.set(Some(chunk.upstream)),
        Some(exp) => assert_eq!(
            chunk.upstream, exp,
            "chunk {i} has upstream {:?}, expected {:?}",
            chunk.upstream, exp
        ),
    }
}

async fn run_and_check(
    stream: impl futures::Stream<Item = AgentCompletionChunk> + Unpin,
) -> AgentCompletion {
    let expected_created = std::cell::Cell::new(None);
    let expected_upstream: std::cell::Cell<Option<objectiveai::agent::Upstream>> =
        std::cell::Cell::new(None);
    let agg = common::stream_harness::consume_stream(
        stream,
        |agg, c| agg.push(c),
        |i, chunk| {
            check_created_and_upstream(&expected_created, &expected_upstream, i, chunk);
            assert!(chunk.messages.len() <= 1, "chunk {i} has {} messages, expected at most 1", chunk.messages.len());
            assert!(chunk.usage.is_none(), "chunk {i} (non-final) has usage, expected None");
            assert!(chunk.continuation.is_none(), "chunk {i} (non-final) has continuation, expected None");
        },
        |i, chunk| {
            check_created_and_upstream(&expected_created, &expected_upstream, i, chunk);
            assert!(chunk.messages.len() <= 1, "chunk {i} has {} messages, expected at most 1", chunk.messages.len());
            assert!(chunk.usage.is_none(), "chunk {i} (non-final) has usage, expected None");
            assert!(chunk.continuation.is_none(), "chunk {i} (non-final) has continuation, expected None");
        },
        |i, chunk| {
            check_created_and_upstream(&expected_created, &expected_upstream, i, chunk);
            assert!(chunk.usage.is_some(), "final chunk {i} has no usage, expected Some");
            assert!(chunk.continuation.is_some(), "final chunk {i} has no continuation, expected Some");
        },
    )
    .await;
    AgentCompletion::from(agg)
}

fn normalize(mut c: AgentCompletion) -> AgentCompletion {
    c.normalize_for_tests();
    c
}

fn assert_snapshot(json: &str, path: &str, expected: &str) {
    common::stream_harness::assert_snapshot(
        json, path, expected,
        "UPDATE_AGENT_COMPLETIONS_CLIENT_TESTS_SNAPSHOTS",
    );
}

/// POST `params` to `/agent/completions` (streaming) on the spawned api
/// server and return the resulting `Stream<AgentCompletionChunk>`.
/// Panics if any chunk fails to deserialize or the stream errors.
async fn post_streaming(
    params: AgentCompletionCreateParams,
) -> impl futures::Stream<Item = AgentCompletionChunk> + Unpin {
    let http = common::server::client();
    let stream = http
        .send_streaming::<AgentCompletionChunk, _, _>(
            reqwest::Method::POST,
            "/agent/completions",
            Some(params),
        )
        .await
        .expect("send_streaming should succeed");
    Box::pin(stream.map(|item| match item {
        Ok(chunk) => chunk,
        Err(e) => panic!("chunk deserialize / stream error: {e:?}"),
    }))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

/// Default mock agent, no error — POC for the integration test pattern.
#[tokio::test]
async fn test_basic_mock_agent_seed_42() {
    let params = AgentCompletionCreateParams {
        messages: vec![],
        agent: objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(
            objectiveai::agent::InlineAgentBaseWithFallbacks {
                inner: objectiveai::agent::InlineAgentBase::Mock(MockAgentBase::default()),
                fallbacks: None,
            },
        ),
        provider: None,
        response_format: None,
        seed: Some(42),
        stream: Some(true),
        continuation: None,
    };

    let stream = post_streaming(params).await;
    let completion = normalize(run_and_check(stream).await);
    let json = serde_json::to_string_pretty(&completion).unwrap();
    assert_snapshot(
        &json,
        concat!(env!("CARGO_MANIFEST_DIR"), "/assets/agent/completions/client_tests/test_basic_mock_agent_seed_42.json"),
        include_str!("../assets/agent/completions/client_tests/test_basic_mock_agent_seed_42.json"),
    );
}
