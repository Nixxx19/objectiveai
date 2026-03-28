//! Integration tests for function execution CLI commands.
//!
//! These tests run the CLI binary against a running API server and compare
//! the output against snapshot files from objectiveai-api/assets/.
//! Skipped if OBJECTIVEAI_TEST_PORT is not set.

use std::path::{Path, PathBuf};
use std::process::Command;

fn snapshots_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../objectiveai-api/assets/functions/executions/client_tests")
}

fn load_snapshot(name: &str) -> serde_json::Value {
    let path = snapshots_dir().join(format!("{name}.json"));
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read snapshot {}: {e}", path.display()));
    serde_json::from_str(&content).unwrap()
}

/// Extract the output field from a snapshot (the full unary response).
fn snapshot_output(snapshot: &serde_json::Value) -> serde_json::Value {
    snapshot["output"]["output"].clone()
}

/// Extract whether the snapshot has task errors.
fn snapshot_has_errors(snapshot: &serde_json::Value) -> bool {
    snapshot["tasks_errors"].as_bool().unwrap_or(false)
}

use std::sync::Once;

static BUILD_ONCE: Once = Once::new();

fn test_target_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../target/test-cli")
}

fn cli_binary() -> PathBuf {
    let target_dir = test_target_dir();
    let mut path = target_dir.join("debug/objectiveai-cli");
    if cfg!(windows) {
        path.set_extension("exe");
    }

    BUILD_ONCE.call_once(|| {
        let status = Command::new("cargo")
            .args([
                "build", "-p", "objectiveai-cli",
                "--no-default-features", "--features", "rustpython",
                "--target-dir", target_dir.to_str().unwrap(),
            ])
            .status()
            .expect("failed to run cargo build");
        assert!(status.success(), "cargo build failed");
    });

    path
}

struct ExecutionTestCase {
    snapshot: &'static str,
    function_name: &'static str,
    profile_name: &'static str,
    input: String,
    seed: i64,
}

/// Run the CLI with function execution create args and parse the JSON output.
fn run_execution(case: &ExecutionTestCase) -> serde_json::Value {
    let mut cmd = Command::new(cli_binary());

    // Use the tests directory as config base so we don't touch the user's real config
    let tests_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    cmd.env("CONFIG_BASE_DIR", &tests_dir);

    cmd.args(["functions", "executions", "create", "standard"]);

    // Function (mock remote)
    cmd.args(["--function-remote", "mock", "--function-name", case.function_name]);

    // Profile (mock remote)
    cmd.args(["--profile-remote", "mock", "--profile-name", case.profile_name]);

    // Input as inline JSON
    cmd.args(["--input-inline", &case.input]);

    // Seed for deterministic responses
    cmd.args(["--seed", &case.seed.to_string()]);

    let output = cmd.output().expect("failed to execute CLI binary");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        panic!(
            "CLI exited with {}\nstdout: {stdout}\nstderr: {stderr}",
            output.status
        );
    }

    serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("failed to parse CLI output as JSON: {e}\nstdout: {stdout}"))
}

/// Round floats to 8 significant figures to match cross-language comparison.
fn rounded(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                // Double-round: 12 sig figs then 8 sig figs
                let s12 = format!("{:.12e}", f);
                let f12: f64 = s12.parse().unwrap_or(f);
                let s8 = format!("{:.8e}", f12);
                let f8: f64 = s8.parse().unwrap_or(f12);
                serde_json::Value::Number(serde_json::Number::from_f64(f8).unwrap_or_else(|| n.clone()))
            } else {
                value.clone()
            }
        }
        serde_json::Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(rounded).collect())
        }
        serde_json::Value::Object(obj) => {
            serde_json::Value::Object(obj.iter().map(|(k, v)| (k.clone(), rounded(v))).collect())
        }
        _ => value.clone(),
    }
}

macro_rules! snapshot_test {
    ($name:ident, $snapshot:expr, $function:expr, $profile:expr, $seed:expr, $input:tt) => {
        #[test]
        fn $name() {
            let case = ExecutionTestCase {
                snapshot: $snapshot,
                function_name: $function,
                profile_name: $profile,
                input: serde_json::to_string(&serde_json::json!($input)).unwrap(),
                seed: $seed,
            };

            let snapshot = load_snapshot(case.snapshot);
            let expected_output = rounded(&snapshot_output(&snapshot));
            let has_errors = snapshot_has_errors(&snapshot);

            let cli_result = run_execution(&case);
            let actual_output = rounded(&cli_result["output"]);

            assert_eq!(actual_output, expected_output, "output mismatch for {}", case.snapshot);

            if has_errors {
                assert!(
                    cli_result.get("errors").is_some_and(|e| e.as_array().is_some_and(|a| !a.is_empty())),
                    "expected errors for {} but got none",
                    case.snapshot
                );
            } else {
                assert!(
                    cli_result.get("errors").is_none() || cli_result["errors"].as_array().is_some_and(|a| a.is_empty()),
                    "expected no errors for {} but got: {:?}",
                    case.snapshot,
                    cli_result.get("errors")
                );
            }
        }
    };
}

// Same test cases as objectiveai-js/objectiveai-py
snapshot_test!(
    mock_1_scalar_leaf_binary_seed_42,
    "mock_1_scalar_leaf_binary_seed_42",
    "binary-classifier",
    "solo-instruction",
    42,
    {"text": "Hello world"}
);

snapshot_test!(
    mock_7_vector_5_criteria_seed_42,
    "mock_7_vector_5_criteria_seed_42",
    "five-criteria-ranker",
    "schema-heavy-trio",
    42,
    {"items": ["Option A", "Option B", "Option C"]}
);

snapshot_test!(
    mock_20_vector_super_branch_seed_42,
    "mock_20_vector_super_branch_seed_42",
    "nested-vector-super-branch",
    "nested-vector-inline-remote",
    42,
    {"items": ["Alpha", "Beta", "Gamma"]}
);
