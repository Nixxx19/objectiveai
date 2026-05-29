//! Formatter for MCP plugin and tool response bodies.
//!
//! When the MCP server runs a CLI plugin or tool, the CLI emits a
//! stream of [`Output`] values that we collect into a `Vec<Output>`
//! and then turn into a single response body via [`format_outputs`].
//! Rendering differs between plugins and tools, and the tool path
//! has a special case for [`NotificationValue::ToolLine`].
//!
//! # Dispatch rules
//!
//! Let `outputs` be the collected `Vec<Output>`. The body is:
//!
//! 1. `outputs.is_empty()` → the literal string `<empty>`.
//! 2. `outputs.len() == 1` AND `outputs[0]` is `Output::Notification(n)`:
//!    - **Plugin** mode → JSON of `n.value` (the bare
//!      `NotificationValue`, dropping the `Notification` envelope and
//!      its `agent_id`).
//!    - **Tool** mode, `ToolLine` with `stderr != Some(true)` (i.e.
//!      stdout, including the ambiguous case where neither flag is
//!      set) → the raw `line` text. **Not** JSON-encoded — the body
//!      is the literal string content.
//!    - **Tool** mode, `ToolLine` with `stderr == Some(true)` → JSON
//!      of the bare [`ToolLine`] struct (no `NotificationValue`
//!      `kind` tag, no `Notification` envelope).
//!    - **Tool** mode, any other `NotificationValue` → JSON of
//!      `n.value` (same as plugin).
//! 3. Otherwise — length > 1, OR length 1 with an `Error` —
//!    → a JSON array whose elements follow the per-element rules
//!    below. The whole array is `serde_json::to_string`-encoded.
//!
//! # Per-element rendering inside the array
//!
//! - `Output::Error` → the full `Output` JSON, retaining the
//!   `"type":"error"` discriminator so consumers can tell errors
//!   apart from notifications in a mixed array.
//! - `Output::Notification(n)`, **Plugin** mode → `n.value`.
//! - `Output::Notification(ToolLine(tl))`, **Tool** mode, stderr →
//!   the bare `ToolLine` struct (no `NotificationValue` wrapper).
//! - `Output::Notification(ToolLine(tl))`, **Tool** mode, stdout →
//!   a JSON string element (the `line` text).
//! - `Output::Notification(other)`, **Tool** mode → `n.value`.
//!
//! # `test_mode`
//!
//! When `test_mode` is `true`, the `agent_id` field is stripped from
//! every rendered `Output::Error` value before serialization. Plugin
//! and tool notification renderings already drop `agent_id` by
//! construction (the `Notification` envelope is unwrapped to its
//! inner `NotificationValue`, which has no `agent_id` field). The
//! bare `ToolLine` likewise has no `agent_id` field. So errors are
//! the only shape that needs an explicit strip.

use objectiveai_sdk::cli::output::{Notification, NotificationValue, Output, ToolLine};
use serde_json::Value;

/// Whether the formatter is rendering a plugin response or a tool
/// response. The tool path has special handling for `ToolLine`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputMode {
    Plugin,
    Tool,
}

/// Body returned for `outputs.is_empty()`. A tool stdout line whose
/// literal text is `<empty>` is indistinguishable from the
/// empty-response case in single-stdout-line mode.
const EMPTY_SENTINEL: &str = "<empty>";

/// Format collected CLI outputs into an MCP tool response body.
/// See the module-level docs for the full dispatch table.
pub fn format_outputs(mode: OutputMode, outputs: &[Output], test_mode: bool) -> String {
    if outputs.is_empty() {
        return EMPTY_SENTINEL.to_string();
    }

    if outputs.len() == 1 {
        if let Output::Notification(n) = &outputs[0] {
            return render_single_notification(mode, n);
        }
    }

    let elems: Vec<Value> = outputs
        .iter()
        .map(|o| render_array_element(mode, o, test_mode))
        .collect();
    serialize_or_fallback(&elems)
}

fn render_single_notification(mode: OutputMode, n: &Notification) -> String {
    match mode {
        OutputMode::Plugin => serialize_or_fallback(&n.value),
        OutputMode::Tool => match &n.value {
            NotificationValue::ToolLine(tl) if is_stderr(tl) => serialize_or_fallback(tl),
            NotificationValue::ToolLine(tl) => tl.line.clone(),
            other => serialize_or_fallback(other),
        },
    }
}

fn render_array_element(mode: OutputMode, output: &Output, test_mode: bool) -> Value {
    match output {
        Output::Error(_) => {
            let mut v = serde_json::to_value(output).unwrap_or(Value::Null);
            if test_mode {
                strip_agent_id(&mut v);
            }
            v
        }
        Output::Notification(n) => match mode {
            OutputMode::Plugin => serde_json::to_value(&n.value).unwrap_or(Value::Null),
            OutputMode::Tool => match &n.value {
                NotificationValue::ToolLine(tl) if is_stderr(tl) => {
                    serde_json::to_value(tl).unwrap_or(Value::Null)
                }
                NotificationValue::ToolLine(tl) => Value::String(tl.line.clone()),
                other => serde_json::to_value(other).unwrap_or(Value::Null),
            },
        },
    }
}

/// `true` only when `stderr == Some(true)`. Everything else
/// (including the SDK-invariant-violating `None`/`None` case) falls
/// back to stdout rendering, because stdout is the lossy path
/// (drops the struct down to a bare string) and should only be
/// taken when the line is unambiguously stdout.
fn is_stderr(tl: &ToolLine) -> bool {
    tl.stderr == Some(true)
}

fn strip_agent_id(v: &mut Value) {
    if let Value::Object(map) = v {
        map.remove("agent_id");
    }
}

fn serialize_or_fallback<T: serde::Serialize>(v: &T) -> String {
    serde_json::to_string(v).unwrap_or_else(|e| format!("error serializing output: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use objectiveai_sdk::cli::output::{Error, Level, Ok as NotifOk};

    fn notif(value: NotificationValue) -> Output {
        Output::Notification(Notification {
            value,
            agent_id: None,
        })
    }

    fn notif_with_agent(value: NotificationValue, agent_id: &str) -> Output {
        Output::Notification(Notification {
            value,
            agent_id: Some(agent_id.to_string()),
        })
    }

    fn err(message: &str) -> Output {
        Output::Error(Error {
            level: Level::Error,
            fatal: false,
            message: serde_json::Value::String(message.to_string()),
            agent_id: None,
        })
    }

    fn err_with_agent(message: &str, agent_id: &str) -> Output {
        Output::Error(Error {
            level: Level::Error,
            fatal: false,
            message: serde_json::Value::String(message.to_string()),
            agent_id: Some(agent_id.to_string()),
        })
    }

    fn stdout_line(line: &str) -> Output {
        notif(NotificationValue::ToolLine(ToolLine {
            line: line.to_string(),
            stdout: Some(true),
            stderr: None,
        }))
    }

    fn stderr_line(line: &str) -> Output {
        notif(NotificationValue::ToolLine(ToolLine {
            line: line.to_string(),
            stdout: None,
            stderr: Some(true),
        }))
    }

    #[test]
    fn empty_returns_sentinel_in_both_modes() {
        assert_eq!(format_outputs(OutputMode::Plugin, &[], false), "<empty>");
        assert_eq!(format_outputs(OutputMode::Tool, &[], false), "<empty>");
    }

    #[test]
    fn plugin_single_notification_emits_bare_notification_value() {
        let outputs = vec![notif(NotificationValue::Ok(NotifOk { ok: true }))];
        let body = format_outputs(OutputMode::Plugin, &outputs, false);
        assert_eq!(body, r#"{"kind":"ok","ok":true}"#);
    }

    #[test]
    fn plugin_single_notification_drops_agent_id_from_envelope() {
        let outputs = vec![notif_with_agent(
            NotificationValue::Ok(NotifOk { ok: true }),
            "agent-x",
        )];
        let body = format_outputs(OutputMode::Plugin, &outputs, false);
        assert!(!body.contains("agent_id"), "got: {body}");
        assert!(!body.contains("agent-x"), "got: {body}");
    }

    #[test]
    fn tool_single_notification_non_toolline_matches_plugin() {
        let outputs = vec![notif(NotificationValue::Ok(NotifOk { ok: true }))];
        let body = format_outputs(OutputMode::Tool, &outputs, false);
        assert_eq!(body, r#"{"kind":"ok","ok":true}"#);
    }

    #[test]
    fn tool_single_toolline_stdout_emits_raw_line() {
        let outputs = vec![stdout_line("hello world")];
        let body = format_outputs(OutputMode::Tool, &outputs, false);
        assert_eq!(body, "hello world");
    }

    #[test]
    fn tool_single_toolline_stderr_emits_bare_toolline_object() {
        let outputs = vec![stderr_line("oops")];
        let body = format_outputs(OutputMode::Tool, &outputs, false);
        let v: Value = serde_json::from_str(&body).expect("parse body");
        assert_eq!(v["line"], "oops");
        assert_eq!(v["stderr"], true);
        assert_eq!(v.get("stdout"), None);
        assert_eq!(v.get("kind"), None, "must not have NotificationValue tag");
    }

    #[test]
    fn plugin_single_toolline_emits_full_notification_value() {
        let outputs = vec![stdout_line("hi")];
        let body = format_outputs(OutputMode::Plugin, &outputs, false);
        let v: Value = serde_json::from_str(&body).expect("parse body");
        assert_eq!(v["kind"], "tool_line", "plugin mode keeps the kind tag");
        assert_eq!(v["line"], "hi");
        assert_eq!(v["stdout"], true);
    }

    #[test]
    fn single_error_uses_array_path() {
        let outputs = vec![err("nope")];
        let body = format_outputs(OutputMode::Plugin, &outputs, false);
        let v: Value = serde_json::from_str(&body).expect("parse body");
        let arr = v.as_array().expect("array");
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["type"], "error");
        assert_eq!(arr[0]["message"], "nope");
    }

    #[test]
    fn plugin_multi_all_notifications_emits_array_of_notification_values() {
        let outputs = vec![
            notif(NotificationValue::Ok(NotifOk { ok: true })),
            notif(NotificationValue::Ok(NotifOk { ok: true })),
        ];
        let body = format_outputs(OutputMode::Plugin, &outputs, false);
        assert_eq!(body, r#"[{"kind":"ok","ok":true},{"kind":"ok","ok":true}]"#);
    }

    #[test]
    fn tool_multi_mixed_stdout_stderr_emits_strings_and_toolline_objects() {
        let outputs = vec![stdout_line("a"), stderr_line("b"), stdout_line("c")];
        let body = format_outputs(OutputMode::Tool, &outputs, false);
        let v: Value = serde_json::from_str(&body).expect("parse body");
        let arr = v.as_array().expect("array");
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0], Value::String("a".into()));
        assert_eq!(arr[1]["line"], "b");
        assert_eq!(arr[1]["stderr"], true);
        assert_eq!(arr[1].get("kind"), None);
        assert_eq!(arr[2], Value::String("c".into()));
    }

    #[test]
    fn multi_with_error_keeps_error_envelope() {
        let outputs = vec![
            notif(NotificationValue::Ok(NotifOk { ok: true })),
            err("boom"),
            notif(NotificationValue::Ok(NotifOk { ok: true })),
        ];
        let body = format_outputs(OutputMode::Plugin, &outputs, false);
        let v: Value = serde_json::from_str(&body).expect("parse body");
        let arr = v.as_array().expect("array");
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0]["kind"], "ok");
        assert_eq!(arr[1]["type"], "error");
        assert_eq!(arr[1]["message"], "boom");
        assert_eq!(arr[2]["kind"], "ok");
    }

    #[test]
    fn test_mode_strips_agent_id_from_errors() {
        let outputs = vec![err_with_agent("nope", "agent-x")];
        let body = format_outputs(OutputMode::Plugin, &outputs, true);
        let v: Value = serde_json::from_str(&body).expect("parse body");
        assert_eq!(v[0]["type"], "error");
        assert_eq!(v[0].get("agent_id"), None);
        assert!(!body.contains("agent-x"), "got: {body}");
    }

    #[test]
    fn non_test_mode_keeps_agent_id_on_errors() {
        let outputs = vec![err_with_agent("nope", "agent-x")];
        let body = format_outputs(OutputMode::Plugin, &outputs, false);
        let v: Value = serde_json::from_str(&body).expect("parse body");
        assert_eq!(v[0]["agent_id"], "agent-x");
    }
}
