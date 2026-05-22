//! Snapshot tests for the CLI driving `vector completions post`.
//!
//! Sibling to `agent_completion_snapshots.rs`. Same fixture-directory
//! convention (`objectiveai-api/assets/vector/completions/client_tests/`).
//!
//! Currently exercises one scenario: a 20-agent mock swarm where every
//! agent declares 10 entries in `client_objectiveai_mcp.tools` (no
//! plugins, `objectiveai` field omitted). Driven through the CLI's
//! `objectiveai api vector completions post --body-inline …` command
//! (`stream: true` so the SSE-stamped CLI path is taken), not via
//! direct SDK / API calls — exercising the full CLI-resolves-config
//! -> CLI-emits-jsonl -> SDK-talks-to-api path.
//!
//! Set `UPDATE_VECTOR_COMPLETIONS_CLIENT_TESTS_SNAPSHOTS=1` to (re)write
//! the snapshot from the current run, matching the API integration
//! suite's convention.

mod cli_test_util;

use std::path::{Path, PathBuf};

fn snapshots_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../objectiveai-api/assets/vector/completions/client_tests")
}

/// The full streaming chunk array is full of wall-clock-derived ids
/// and `created` timestamps that change every run. Reduce to the
/// determinism-bearing fields only: the final chunk's `scores`,
/// `weights`, content-hashed `swarm` id, and the total count of
/// agent completions across all chunks (a regression in count
/// expansion would surface here).
fn distill(chunks: &serde_json::Value) -> serde_json::Value {
    let arr = chunks.as_array().expect("expected array of chunks");
    let last = arr.last().expect("expected at least one chunk");
    let total_completions: usize = arr
        .iter()
        .filter_map(|c| c.get("completions").and_then(|v| v.as_array()).map(|a| a.len()))
        .sum();
    serde_json::json!({
        "scores": last.get("scores").cloned().unwrap_or(serde_json::Value::Null),
        "weights": last.get("weights").cloned().unwrap_or(serde_json::Value::Null),
        "swarm": last.get("swarm").cloned().unwrap_or(serde_json::Value::Null),
        "total_completions": total_completions,
    })
}

fn assert_snapshot(actual: &serde_json::Value, name: &str) {
    let path = snapshots_dir().join(format!("{name}.json"));
    let actual_str = serde_json::to_string_pretty(actual).unwrap();
    if std::env::var("UPDATE_VECTOR_COMPLETIONS_CLIENT_TESTS_SNAPSHOTS").as_deref()
        == Ok("1")
    {
        std::fs::write(&path, format!("{actual_str}\n")).unwrap();
        eprintln!("Updated snapshot: {}", path.display());
        return;
    }
    let expected_str = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read snapshot {}: {e}", path.display()));
    assert_eq!(
        actual_str,
        expected_str.trim_end(),
        "snapshot mismatch for {}",
        path.display(),
    );
}

#[test]
fn test_twenty_agents_10x_tools_seed_42() {
    if cli_test_util::test_api_address().is_none() {
        eprintln!(
            "OBJECTIVEAI_TEST_PORT not set — skipping test_twenty_agents_10x_tools_seed_42"
        );
        return;
    }

    let body = serde_json::json!({
        "messages": [{"role": "user", "content": "choose A or B"}],
        "responses": ["A", "B"],
        "swarm": {"remote": "mock", "name": "twenty-agents-10x-tools"},
        "seed": 42,
        // Streaming so the CLI takes the `send_streaming` path that
        // stamps `X-Transport: sse` — the API's `/vector/completions`
        // route only accepts that branch over HTTP (the other branch
        // is the WS upgrade).
        "stream": true,
    })
    .to_string();

    let chunks = cli_test_util::run_cli(&[
        "api", "vector", "completions", "post",
        "--body-inline", &body,
    ]);

    let distilled = distill(&chunks);
    assert_snapshot(&distilled, "twenty_agents_10x_tools_seed_42");
}
