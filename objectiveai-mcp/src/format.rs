//! Formatter for MCP tool response bodies.
//!
//! When the MCP server runs a CLI plugin or tool, the CLI emits a
//! stream of [`Output`] values that we collect into a `Vec<Output>`
//! and then turn into a single response body via [`format_outputs`].
//! There is exactly one rendering mode: [`NotificationValue::ToolLine`]
//! stdout is unwrapped to the raw `line` text, `ToolLine` stderr is
//! unwrapped to the bare struct (no `NotificationValue` envelope),
//! and every other notification is rendered as full `NotificationValue`
//! JSON.
//!
//! # Dispatch rules
//!
//! Let `outputs` be the collected `Vec<Output>`. The body is:
//!
//! 1. `outputs.is_empty()` → the literal string `<empty>`.
//! 2. `outputs.len() == 1` AND `outputs[0]` is `Output::Notification(n)`:
//!    - `ToolLine` with `stderr == Some(true)` → JSON of the bare
//!      [`ToolLine`] struct (no `NotificationValue` `kind` tag, no
//!      `Notification` envelope).
//!    - `ToolLine` with `stderr != Some(true)` (i.e. stdout,
//!      including the ambiguous case where neither flag is set) →
//!      the raw `line` text. **Not** JSON-encoded — the body is
//!      the literal string content.
//!    - any other `NotificationValue` → JSON of `n.value` (the bare
//!      `NotificationValue`, dropping the `Notification` envelope
//!      and its `agent_id`).
//! 3. Otherwise — length > 1, OR length 1 with an `Error` —
//!    → a JSON array whose elements follow the per-element rules
//!    below. The whole array is `serde_json::to_string`-encoded.
//!
//! # Per-element rendering inside the array
//!
//! - `Output::Error` → the full `Output` JSON (`{"type":"error",…}`),
//!   with `agent_id` always set to `None` before serialization so the
//!   `skip_serializing_if` drops it from the wire shape.
//! - `Output::Notification(ToolLine(tl))`, stderr → the bare
//!   `ToolLine` struct (no `NotificationValue` wrapper).
//! - `Output::Notification(ToolLine(tl))`, stdout → a JSON string
//!   element (the `line` text).
//! - `Output::Notification(other)` → `n.value`.
//!
//! # `agent_id` stripping
//!
//! The formatter unconditionally drops `agent_id` from every rendered
//! shape. Notification renderings drop it for free — the formatter
//! unwraps `Notification` to its inner `NotificationValue`, which has
//! no `agent_id` field. The bare `ToolLine` likewise has no
//! `agent_id` field. For `Output::Error` the `agent_id` field is
//! explicitly cleared on a clone before serialization
//! (`skip_serializing_if = "Option::is_none"` then omits it).

use objectiveai_sdk::cli::output::{Error, Notification, NotificationValue, Output, ToolLine};
use rmcp::model::Content;
use serde::Serialize;

/// Body returned for `outputs.is_empty()`. A tool stdout line whose
/// literal text is `<empty>` is indistinguishable from the
/// empty-response case in single-stdout-line mode.
const EMPTY_SENTINEL: &str = "<empty>";

/// Per-element shape in the array path. `#[serde(untagged)]`: each
/// variant serializes as just its inner value, with no enum-level
/// discriminator. The owned `ErrorOutput` variant exists so we can
/// serialize an `Error` clone whose `agent_id` has been cleared.
#[derive(Serialize)]
#[serde(untagged)]
enum RenderedElement<'a> {
    Notification(&'a NotificationValue),
    Line(&'a str),
    ToolLine(&'a ToolLine),
    ErrorOutput(Output),
}

/// Format collected CLI outputs into the MCP tool response `Content`
/// vector. Today this is always a single `Content::text` carrying
/// the JSON-encoded body produced by [`render_body`]; the
/// `Vec<Content>` shape is set up so a follow-up change can start
/// emitting multiple/typed blocks (e.g. native MCP `image` or
/// `audio` for typed `LogContent` notifications) without touching
/// every call site again.
///
/// See the module-level docs for the full per-element dispatch
/// table; the rules live in [`render_body`].
pub fn format_outputs(outputs: &[Output]) -> Vec<Content> {
    vec![Content::text(render_body(outputs))]
}

/// Internal: walk the dispatch table and produce the JSON-encoded
/// response body. Kept separate from [`format_outputs`] so the
/// `format::tests` suite can keep asserting on the literal string
/// shape — and so the upcoming media-routing follow-up can call
/// this for the text fallback case while emitting native blocks
/// for typed `LogContent` notifications alongside.
fn render_body(outputs: &[Output]) -> String {
    if outputs.is_empty() {
        return EMPTY_SENTINEL.to_string();
    }

    if outputs.len() == 1 {
        if let Output::Notification(n) = &outputs[0] {
            return render_single_notification(n);
        }
    }

    let elems: Vec<RenderedElement<'_>> = outputs
        .iter()
        .map(render_array_element)
        .collect();
    serialize_or_fallback(&elems)
}

fn render_single_notification(n: &Notification) -> String {
    match &n.value {
        NotificationValue::ToolLine(tl) if is_stderr(tl) => serialize_or_fallback(tl),
        NotificationValue::ToolLine(tl) => tl.line.clone(),
        other => serialize_or_fallback(other),
    }
}

fn render_array_element(output: &Output) -> RenderedElement<'_> {
    match output {
        Output::Error(e) => RenderedElement::ErrorOutput(Output::Error(strip_error_agent_id(e))),
        Output::Notification(n) => match &n.value {
            NotificationValue::ToolLine(tl) if is_stderr(tl) => RenderedElement::ToolLine(tl),
            NotificationValue::ToolLine(tl) => RenderedElement::Line(&tl.line),
            other => RenderedElement::Notification(other),
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

fn strip_error_agent_id(e: &Error) -> Error {
    let mut stripped = e.clone();
    stripped.agent_id = None;
    stripped
}

fn serialize_or_fallback<T: Serialize>(v: &T) -> String {
    serde_json::to_string(v).unwrap_or_else(|e| format!("error serializing output: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use objectiveai_sdk::cli::output::{Level, Ok as NotifOk};
    use serde_json::Value;

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
            message: Value::String(message.to_string()),
            agent_id: None,
        })
    }

    fn err_with_agent(message: &str, agent_id: &str) -> Output {
        Output::Error(Error {
            level: Level::Error,
            fatal: false,
            message: Value::String(message.to_string()),
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
    fn empty_returns_sentinel() {
        assert_eq!(render_body(&[]), "<empty>");
    }

    #[test]
    fn single_notification_non_toolline_emits_bare_notification_value() {
        let outputs = vec![notif(NotificationValue::Ok(NotifOk { ok: true }))];
        let body = render_body(&outputs);
        assert_eq!(body, r#"{"kind":"ok","ok":true}"#);
    }

    #[test]
    fn single_notification_drops_agent_id_from_envelope() {
        let outputs = vec![notif_with_agent(
            NotificationValue::Ok(NotifOk { ok: true }),
            "agent-x",
        )];
        let body = render_body(&outputs);
        assert!(!body.contains("agent_id"), "got: {body}");
        assert!(!body.contains("agent-x"), "got: {body}");
    }

    #[test]
    fn single_toolline_stdout_emits_raw_line() {
        let outputs = vec![stdout_line("hello world")];
        let body = render_body(&outputs);
        assert_eq!(body, "hello world");
    }

    #[test]
    fn single_toolline_stderr_emits_bare_toolline_object() {
        let outputs = vec![stderr_line("oops")];
        let body = render_body(&outputs);
        let v: Value = serde_json::from_str(&body).expect("parse body");
        assert_eq!(v["line"], "oops");
        assert_eq!(v["stderr"], true);
        assert_eq!(v.get("stdout"), None);
        assert_eq!(v.get("kind"), None, "must not have NotificationValue tag");
    }

    #[test]
    fn single_error_uses_array_path() {
        let outputs = vec![err("nope")];
        let body = render_body(&outputs);
        let v: Value = serde_json::from_str(&body).expect("parse body");
        let arr = v.as_array().expect("array");
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["type"], "error");
        assert_eq!(arr[0]["message"], "nope");
    }

    #[test]
    fn multi_mixed_stdout_stderr_emits_strings_and_toolline_objects() {
        let outputs = vec![stdout_line("a"), stderr_line("b"), stdout_line("c")];
        let body = render_body(&outputs);
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
        let body = render_body(&outputs);
        let v: Value = serde_json::from_str(&body).expect("parse body");
        let arr = v.as_array().expect("array");
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0]["kind"], "ok");
        assert_eq!(arr[1]["type"], "error");
        assert_eq!(arr[1]["message"], "boom");
        assert_eq!(arr[2]["kind"], "ok");
    }

    #[test]
    fn agent_id_is_always_stripped_from_errors() {
        let outputs = vec![err_with_agent("nope", "agent-x")];
        let body = render_body(&outputs);
        let v: Value = serde_json::from_str(&body).expect("parse body");
        assert_eq!(v[0]["type"], "error");
        assert_eq!(v[0].get("agent_id"), None);
        assert!(!body.contains("agent-x"), "got: {body}");
    }

    #[test]
    fn agent_id_is_always_stripped_from_errors_in_mixed_array() {
        let outputs = vec![
            notif_with_agent(NotificationValue::Ok(NotifOk { ok: true }), "agent-y"),
            err_with_agent("boom", "agent-z"),
        ];
        let body = render_body(&outputs);
        assert!(!body.contains("agent_id"), "got: {body}");
        assert!(!body.contains("agent-y"), "got: {body}");
        assert!(!body.contains("agent-z"), "got: {body}");
    }
}
