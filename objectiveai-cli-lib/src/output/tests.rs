use super::*;
use serde::{Deserialize, Serialize};
use serde_json::json;

fn roundtrip<T>(out: &Output<T>) -> serde_json::Value
where
    T: Serialize + serde::de::DeserializeOwned,
{
    let s = serde_json::to_string(out).unwrap();
    let back: Output<T> = serde_json::from_str(&s).unwrap();
    serde_json::to_value(&back).unwrap()
}

#[tokio::test]
async fn emit_with_none_handle_writes_to_stdout() {
    // Smoke test that emit(&None) runs without panicking. We can't
    // easily intercept stdout from a unit test, so just confirm the
    // call completes and the future is Send + 'static-safe.
    let out: Output<Ok> = Output::Notification(OK);
    out.emit(&None).await;
}

#[test]
fn error_fatal_wire_shape() {
    let out: Output<Ok> = Output::Error(Error {
        level: Level::Error,
        fatal: true,
        message: "favorite not found: foo".to_string(),
    });
    let v = roundtrip(&out);
    assert_eq!(v["type"], "error");
    assert_eq!(v["level"], "error");
    assert_eq!(v["fatal"], true);
    assert_eq!(v["message"], "favorite not found: foo");
}

#[test]
fn error_non_fatal_warn_wire_shape() {
    let out: Output<Ok> = Output::Error(Error {
        level: Level::Warn,
        fatal: false,
        message: "auto-update error: ...".to_string(),
    });
    let v = roundtrip(&out);
    assert_eq!(v["type"], "error");
    assert_eq!(v["level"], "warn");
    assert_eq!(v["fatal"], false);
}

#[test]
fn ack_ok_wire_shape() {
    let out = Output::Notification(OK);
    let v = roundtrip(&out);
    assert_eq!(v["type"], "notification");
    assert_eq!(v["ok"], true);
}

#[test]
fn cleared_wire_shape() {
    let out = Output::Notification(Cleared { cleared: 7 });
    let v = roundtrip(&out);
    assert_eq!(v["type"], "notification");
    assert_eq!(v["cleared"], 7);
}

#[test]
fn instructions_wire_shape() {
    let out = Output::Notification(Instructions {
        instructions: "follow these steps".to_string(),
    });
    let v = roundtrip(&out);
    assert_eq!(v["type"], "notification");
    assert_eq!(v["instructions"], "follow these steps");
}

#[test]
fn items_generic_wire_shape() {
    #[derive(Serialize, Deserialize, Debug)]
    struct Sample {
        n: u32,
    }
    let out = Output::Notification(Items {
        items: vec![Sample { n: 1 }, Sample { n: 2 }],
    });
    let v = roundtrip(&out);
    assert_eq!(v["type"], "notification");
    assert_eq!(v["items"][0]["n"], 1);
    assert_eq!(v["items"][1]["n"], 2);
}

#[test]
fn pair_list_item_untagged_dispatch() {
    // A Favorite-shaped pair list item should deserialize without an
    // explicit tag, by virtue of `#[serde(untagged)]`.
    let item: PairListItem = serde_json::from_value(json!({
        "name": "fav",
        "function": {"remote": "github", "owner": "o", "repository": "r", "commit": "c"},
        "profile":  {"remote": "github", "owner": "o", "repository": "r", "commit": "c"},
        "note": ""
    }))
    .unwrap();
    let out = Output::Notification(Items { items: vec![item] });
    let v = roundtrip(&out);
    assert_eq!(v["items"][0]["name"], "fav");
}

#[test]
fn jq_results_wire_shape() {
    let out = Output::Notification(JqResults {
        jq: json!([{"a": 1}, {"b": 2}]),
    });
    let v = roundtrip(&out);
    assert_eq!(v["type"], "notification");
    assert_eq!(v["jq"][0]["a"], 1);
}

#[test]
fn log_content_json_wire_shape() {
    let out = Output::Notification(LogContent::Json {
        content: json!({"completion": {"id": "abc"}}),
    });
    let v = roundtrip(&out);
    assert_eq!(v["type"], "notification");
    assert_eq!(v["content"]["completion"]["id"], "abc");
}

#[test]
fn log_content_data_url_wire_shape() {
    let out = Output::Notification(LogContent::DataUrl {
        content_data_url: "data:image/png;base64,abc".to_string(),
    });
    let v = roundtrip(&out);
    assert_eq!(v["type"], "notification");
    assert_eq!(v["content_data_url"], "data:image/png;base64,abc");
}

#[test]
fn log_stream_ready_wire_shape() {
    let out = Output::Notification(LogStreamReady {
        log_stream_ready: "abc-123".to_string(),
    });
    let v = roundtrip(&out);
    assert_eq!(v["type"], "notification");
    assert_eq!(v["log_stream_ready"], "abc-123");
}

#[test]
fn published_wire_shape() {
    let out = Output::Notification(Published {
        sha: "deadbeef".to_string(),
    });
    let v = roundtrip(&out);
    assert_eq!(v["type"], "notification");
    assert_eq!(v["sha"], "deadbeef");
}

#[test]
fn schema_wire_shape() {
    let out = Output::Notification(Schema {
        schema: json!({"$schema": "...", "type": "object"}),
    });
    let v = roundtrip(&out);
    assert_eq!(v["type"], "notification");
    assert_eq!(v["schema"]["$schema"], "...");
    // `type` inside the schema must not collide with the outer `type`
    // tag — they're at different JSON-object levels.
    assert_eq!(v["schema"]["type"], "object");
}

#[test]
fn schemas_list_wire_shape() {
    let out = Output::Notification(Schemas {
        schemas: vec!["Foo".to_string(), "Bar".to_string()],
    });
    let v = roundtrip(&out);
    assert_eq!(v["type"], "notification");
    assert_eq!(v["schemas"][0], "Foo");
    assert_eq!(v["schemas"][1], "Bar");
}

#[test]
fn value_generic_wire_shape() {
    let out = Output::Notification(Value {
        value: vec!["a".to_string(), "b".to_string()],
    });
    let v = roundtrip(&out);
    assert_eq!(v["type"], "notification");
    assert_eq!(v["value"][0], "a");
}

#[test]
fn detached_wire_shape() {
    let out = Output::Notification(Detached { pid: 12345 });
    let v = roundtrip(&out);
    assert_eq!(v["type"], "notification");
    assert_eq!(v["pid"], 12345);
}

#[test]
fn error_path_root_serializes_as_string() {
    use crate::output::ErrorPath;
    let s = serde_json::to_string(&ErrorPath::Root).unwrap();
    assert_eq!(s, "\"root\"");
}

#[test]
fn error_path_task_serializes_as_array() {
    use crate::output::ErrorPath;
    let s = serde_json::to_string(&ErrorPath::Task(vec![0, 3, 1])).unwrap();
    assert_eq!(s, "[0,3,1]");
}

#[test]
fn error_path_roundtrips() {
    use crate::output::ErrorPath;
    let cases = [
        ("\"root\"", ErrorPath::Root),
        ("\"reasoning\"", ErrorPath::Reasoning),
        ("[1,2,3]", ErrorPath::Task(vec![1, 2, 3])),
    ];
    for (wire, expected) in cases {
        let parsed: ErrorPath = serde_json::from_str(wire).unwrap();
        match (parsed, expected) {
            (ErrorPath::Root, ErrorPath::Root) | (ErrorPath::Reasoning, ErrorPath::Reasoning) => {}
            (ErrorPath::Task(a), ErrorPath::Task(b)) => assert_eq!(a, b),
            (a, b) => panic!("mismatch: parsed {a:?}, expected {b:?}"),
        }
    }
}
