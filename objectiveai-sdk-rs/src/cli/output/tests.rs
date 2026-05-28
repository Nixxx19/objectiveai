//! Round-trip tests for every [`NotificationValue`] variant plus
//! [`Output::Error`] and the [`Handle`] emit paths.
//!
//! Each per-variant test builds a representative value, serializes it
//! through `Output::Notification`, parses the JSON back, and asserts
//! the deserialized value equals the original via `PartialEq`.

use std::sync::Arc;

use super::*;
use crate::cli::output::notification::{SkipReason, Updater};
use serde_json::json;
use tokio::sync::Mutex;

fn roundtrip(out: &Output) -> Output {
    let s = serde_json::to_string(out).expect("Output serializes");
    serde_json::from_str(&s).expect("Output deserializes")
}

fn notif(value: NotificationValue) -> Output {
    Output::Notification(Notification { value, agent_id: None })
}

fn assert_roundtrip_eq(out: Output) {
    let back = roundtrip(&out);
    assert_eq!(out, back, "round-trip changed shape");
}

#[tokio::test]
async fn emit_via_stdout_handle() {
    // Smoke test that the default Stdout-destination handle routes
    // emit() without panicking. We can't intercept stdout from a unit
    // test, so just confirm the call completes.
    notif(NotificationValue::Ok(OK)).emit(&Handle::stdout()).await;
}

#[tokio::test]
async fn emit_via_collect_handle_appends_to_vec() {
    let vec = Arc::new(Mutex::new(Vec::new()));
    let handle = Handle::from(HandleDestination::Collect(vec.clone()));

    notif(NotificationValue::Ok(OK)).emit(&handle).await;
    Output::Error(Error {
        level: Level::Warn,
        fatal: false,
        message: "heads up".into(),
        agent_id: None,
    })
    .emit(&handle)
    .await;

    let snapshot = vec.lock().await;
    assert_eq!(snapshot.len(), 2);

    let first = serde_json::to_value(&snapshot[0]).unwrap();
    assert_eq!(first["type"], "notification");
    assert_eq!(first["value"]["kind"], "ok");
    assert_eq!(first["value"]["ok"], true);

    let second = serde_json::to_value(&snapshot[1]).unwrap();
    assert_eq!(second["type"], "error");
    assert_eq!(second["level"], "warn");
    assert_eq!(second["fatal"], false);
    assert_eq!(second["message"], "heads up");
}

#[test]
fn error_fatal_roundtrip() {
    let out = Output::Error(Error {
        level: Level::Error,
        fatal: true,
        message: "favorite not found: foo".into(),
        agent_id: None,
    });
    assert_roundtrip_eq(out);
}

#[test]
fn error_non_fatal_warn_roundtrip() {
    let out = Output::Error(Error {
        level: Level::Warn,
        fatal: false,
        message: json!({"code": "x", "detail": [1, 2, 3]}),
        agent_id: Some("cli".to_string()),
    });
    assert_roundtrip_eq(out);
}

// === Per-variant NotificationValue round-trip tests ===

#[test]
fn nv_active_agent_roundtrip() {
    let out = notif(NotificationValue::ActiveAgent(ActiveAgent {
        agent_id: "child-1".into(),
        last_log: 1_700_000_000,
    }));
    assert_roundtrip_eq(out);
}

#[test]
fn nv_agent_items_roundtrip() {
    let out = notif(NotificationValue::AgentItems(AgentItems {
        agent_id: "agent-1".into(),
        items: vec![],
    }));
    assert_roundtrip_eq(out);
}

#[test]
fn nv_spawned_roundtrip() {
    let out = notif(NotificationValue::Spawned(Spawned {
        agent_id: "spawn-xyz".into(),
    }));
    assert_roundtrip_eq(out);
}

#[test]
fn nv_detached_roundtrip() {
    let out = notif(NotificationValue::Detached(Detached { pid: 12345 }));
    assert_roundtrip_eq(out);
}

#[test]
fn nv_inventions_roundtrip() {
    let out = notif(NotificationValue::Inventions(Inventions {
        inventions: vec![InventionResultItem {
            name: "alpha_scalar_leaf".into(),
            path: None,
        }],
    }));
    assert_roundtrip_eq(out);
}

#[test]
fn nv_cleared_roundtrip() {
    let out = notif(NotificationValue::Cleared(Cleared { cleared: 7 }));
    assert_roundtrip_eq(out);
}

#[test]
fn nv_help_roundtrip() {
    let out = notif(NotificationValue::Help(Help {
        help: "Usage: objectiveai [OPTIONS]".into(),
    }));
    assert_roundtrip_eq(out);
}

#[test]
fn nv_installed_roundtrip() {
    let out = notif(NotificationValue::Installed(Installed { installed: true }));
    assert_roundtrip_eq(out);
}

#[test]
fn nv_instructions_roundtrip() {
    let out = notif(NotificationValue::Instructions(Instructions {
        instructions: "# Setup\n\n…".into(),
    }));
    assert_roundtrip_eq(out);
}

#[test]
fn nv_jq_results_roundtrip() {
    let out = notif(NotificationValue::JqResults(JqResults {
        jq: json!([{"a": 1}, {"b": 2}]),
    }));
    assert_roundtrip_eq(out);
}

#[test]
fn nv_log_content_json_roundtrip() {
    let out = notif(NotificationValue::LogContent(LogContent::Json {
        content: json!({"completion": {"id": "abc"}}),
    }));
    assert_roundtrip_eq(out);
}

#[test]
fn nv_log_content_data_url_roundtrip() {
    let out = notif(NotificationValue::LogContent(LogContent::DataUrl {
        content_data_url: "data:image/png;base64,abc".into(),
    }));
    assert_roundtrip_eq(out);
}

#[test]
fn nv_log_stream_ready_roundtrip() {
    let out = notif(NotificationValue::LogStreamReady(LogStreamReady {
        log_stream_ready: "abc-123".into(),
    }));
    assert_roundtrip_eq(out);
}

#[test]
fn nv_ok_roundtrip() {
    let out = notif(NotificationValue::Ok(OK));
    assert_roundtrip_eq(out);
}

#[test]
fn nv_published_roundtrip() {
    let out = notif(NotificationValue::Published(Published {
        sha: "deadbeef".into(),
    }));
    assert_roundtrip_eq(out);
}

#[test]
fn nv_schema_roundtrip() {
    let out = notif(NotificationValue::Schema(Schema {
        schema: json!({"$schema": "...", "type": "object"}),
    }));
    assert_roundtrip_eq(out);
}

#[test]
fn nv_schemas_roundtrip() {
    let out = notif(NotificationValue::Schemas(Schemas {
        schemas: vec!["Foo".into(), "Bar".into()],
    }));
    assert_roundtrip_eq(out);
}

#[test]
fn nv_tool_line_stdout_roundtrip() {
    let out = notif(NotificationValue::ToolLine(ToolLine {
        line: "hello".into(),
        stdout: Some(true),
        stderr: None,
    }));
    assert_roundtrip_eq(out);
}

#[test]
fn nv_tool_line_stderr_roundtrip() {
    let out = notif(NotificationValue::ToolLine(ToolLine {
        line: "oops".into(),
        stdout: None,
        stderr: Some(true),
    }));
    assert_roundtrip_eq(out);
}

#[test]
fn nv_plugins_empty_roundtrip() {
    let out = notif(NotificationValue::Plugins(Plugins { plugins: vec![] }));
    assert_roundtrip_eq(out);
}

#[test]
fn nv_plugin_none_roundtrip() {
    let out = notif(NotificationValue::Plugin(Plugin { plugin: None }));
    assert_roundtrip_eq(out);
}

#[test]
fn nv_tools_empty_roundtrip() {
    let out = notif(NotificationValue::Tools(Tools { tools: vec![] }));
    assert_roundtrip_eq(out);
}

#[test]
fn nv_tool_none_roundtrip() {
    let out = notif(NotificationValue::Tool(Tool { tool: None }));
    assert_roundtrip_eq(out);
}

#[test]
fn nv_updater_checking_roundtrip() {
    let out = notif(NotificationValue::Updater(Updater::Checking {
        asset_name: "objectiveai-x86_64-linux".into(),
        current_version: "1.0.0".into(),
    }));
    assert_roundtrip_eq(out);
}

#[test]
fn nv_updater_skipped_roundtrip() {
    let out = notif(NotificationValue::Updater(Updater::Skipped {
        reason: SkipReason::DevTree,
    }));
    assert_roundtrip_eq(out);
}

#[test]
fn nv_updater_up_to_date_roundtrip() {
    let out = notif(NotificationValue::Updater(Updater::UpToDate {
        current_version: "1.0.0".into(),
        remote_version: "1.0.0".into(),
    }));
    assert_roundtrip_eq(out);
}

#[test]
fn nv_updater_found_roundtrip() {
    let out = notif(NotificationValue::Updater(Updater::Found {
        current_version: "1.0.0".into(),
        remote_version: "1.1.0".into(),
        asset_name: "asset.tar.gz".into(),
        url: "https://example.com/asset.tar.gz".into(),
    }));
    assert_roundtrip_eq(out);
}

#[test]
fn nv_updater_installed_roundtrip() {
    let out = notif(NotificationValue::Updater(Updater::Installed {
        current_version: "1.0.0".into(),
        remote_version: "1.1.0".into(),
    }));
    assert_roundtrip_eq(out);
}

#[test]
fn nv_viewer_send_result_roundtrip() {
    let out = notif(NotificationValue::ViewerSendResult(ViewerSendResult {
        status: 200,
        body: json!({"ok": true}),
    }));
    assert_roundtrip_eq(out);
}

#[test]
fn nv_other_items_roundtrip() {
    // The catch-all: an `Items<T>` payload routes through Other.
    let payload = Items {
        items: vec!["a".to_string(), "b".to_string()],
    };
    let out = notif(NotificationValue::other(&payload));
    assert_roundtrip_eq(out);
}

#[test]
fn nv_other_value_roundtrip() {
    // The catch-all: a `Value<V>` payload routes through Other.
    let payload = Value {
        value: vec![1u32, 2, 3],
    };
    let out = notif(NotificationValue::other(&payload));
    assert_roundtrip_eq(out);
}

#[test]
fn nv_other_raw_object_roundtrip() {
    let payload = json!({"arbitrary": {"nested": [1, 2, 3]}, "kind_hint": null});
    let out = notif(NotificationValue::other(&payload));
    assert_roundtrip_eq(out);
}

#[test]
fn full_envelope_with_agent_id_roundtrip() {
    let out = Output::Notification(Notification {
        agent_id: Some("cli".to_string()),
        value: NotificationValue::Spawned(Spawned {
            agent_id: "x".into(),
        }),
    });
    assert_roundtrip_eq(out);
}

#[test]
fn other_keys_flatten_alongside_kind() {
    // Sanity check: the catch-all variant's map keys land at the same
    // level as `kind`, not nested under a wrapper field.
    let out = notif(NotificationValue::other(&json!({"foo": 1, "bar": "baz"})));
    let v = serde_json::to_value(&out).unwrap();
    assert_eq!(v["value"]["kind"], "other");
    assert_eq!(v["value"]["foo"], 1);
    assert_eq!(v["value"]["bar"], "baz");
}

#[test]
fn typed_variant_carries_kind_discriminator() {
    let out = notif(NotificationValue::Spawned(Spawned {
        agent_id: "abc".into(),
    }));
    let v = serde_json::to_value(&out).unwrap();
    assert_eq!(v["value"]["kind"], "spawned");
    assert_eq!(v["value"]["agent_id"], "abc");
}
