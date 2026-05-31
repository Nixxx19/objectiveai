//! Streaming JSON-array formatter for MCP tool responses.
//!
//! Mirrors `InputValue::to_rich_content_parts`
//! (`agent/completions/message/input_value.rs:402`): build a
//! JSON-shaped sequence piecewise by emitting one MCP `Content`
//! block at a time, using `json_escape::escape_str` to quote-escape
//! any raw string we drop into the output, and flanking media
//! blocks with `"` text blocks so the concatenated text view of the
//! response still reads as a JSON document interleaved with media.
//!
//! # Pipeline
//!
//! Every cli `Output` projects to one or more `RichContentPart`s
//! via either `RichContentPart::from_text_or_data_url(s)` (for
//! string-shaped outputs — `ToolLine`, `PluginNotification` string
//! payload, errors, generic notifications) or the dedicated
//! `From<LogContent>` impl (for typed log content). Each
//! `RichContentPart` then routes to MCP `Content`:
//!
//! - `Text { text }` → one quoted-string text block
//!   (`Content::text("\"<escaped>\"")`).
//! - any media variant → `ContentBlock::from(part)` → bridge to
//!   `rmcp::Content` → flanked by `"` text blocks so the strip-media
//!   view of the response still reads as a JSON-array-of-strings.
//!
//! # Output shape
//!
//! The returned `Vec<Content>` always concatenates (as text) to a
//! JSON-array literal: `[<elem>, <elem>, ..., <elem>]`. Each
//! element is one of:
//!
//! - A single quoted text block.
//! - A three-block sequence (`"`, media `Content`, `"`).
//!
//! Strip the media blocks (and the `"` blocks flanking each one)
//! and the result is a **valid JSON array of strings** — the
//! property the test suite locks in.
//!
//! # `kind` stripping
//!
//! Generic `NotificationValue`s flow through `nv_json_without_kind`
//! before reaching `from_text_or_data_url`. The `kind` field is an
//! internal-cli wire convenience — MCP consumers don't need it.
//!
//! # `agent_id` stripping
//!
//! Notifications are serialized via `n.value` (the inner
//! `NotificationValue`), which has no `agent_id` field. `ToolLine`
//! has none. `Output::Error` is explicitly cleared on a clone
//! before serialization.

use objectiveai_sdk::agent::completions::message::RichContentPart;
use objectiveai_sdk::cli::output::{Error, NotificationValue, Output, ToolLine};
use objectiveai_sdk::filesystem::logs::LogContent;
use objectiveai_sdk::mcp::tool::ContentBlock;
use rmcp::model::Content;
use serde_json::Value;

use crate::bridge::into_rmcp_content;

/// Sentinel returned for `outputs.is_empty()`. Same value the
/// previous formatter used; an MCP client reading the response as
/// text gets `<empty>` rather than `[]`.
const EMPTY_SENTINEL: &str = "<empty>";

/// Format collected CLI outputs into the MCP tool response
/// `Vec<Content>`. See the module-level docs for the dispatch
/// table and the JSON-array-of-strings invariant.
pub fn format_outputs(outputs: &[Output]) -> Vec<Content> {
    if outputs.is_empty() {
        return vec![Content::text(EMPTY_SENTINEL)];
    }

    // Capacity heuristic: each output emits one block; media emit
    // three. Overshoot slightly to avoid reallocs.
    let mut blocks: Vec<Content> = Vec::with_capacity(outputs.len() * 3 + 2);
    blocks.push(Content::text("["));
    let mut first = true;
    for output in outputs {
        if !first {
            blocks.push(Content::text(", "));
        }
        first = false;
        emit_one(output, &mut blocks);
    }
    blocks.push(Content::text("]"));
    blocks
}

/// Append one logical array element to `blocks`. Computes the
/// `RichContentPart` for this output and routes Text → one quoted
/// text block, media → three-block sequence.
fn emit_one(output: &Output, blocks: &mut Vec<Content>) {
    let part = output_to_rich_content_part(output);
    push_part(part, blocks);
}

/// Project one cli `Output` to one `RichContentPart`. String-shaped
/// outputs go through `RichContentPart::from_text_or_data_url` (so
/// payloads that happen to be data URLs are routed to media);
/// `LogContent` rides through the dedicated `From` impl.
fn output_to_rich_content_part(output: &Output) -> RichContentPart {
    match output {
        Output::Error(e) => {
            let stripped = Output::Error(strip_error_agent_id(e));
            let body = serde_json::to_string(&stripped)
                .unwrap_or_else(|_| String::from("<serialize error>"));
            RichContentPart::from_text_or_data_url(body)
        }
        Output::Notification(n) => match &n.value {
            NotificationValue::ToolLine(tl) if is_stderr(tl) => {
                let body = serde_json::to_string(tl)
                    .unwrap_or_else(|_| String::from("<serialize error>"));
                RichContentPart::from_text_or_data_url(body)
            }
            NotificationValue::ToolLine(tl) => {
                RichContentPart::from_text_or_data_url(tl.line.clone())
            }
            NotificationValue::PluginNotification { value: Value::String(s) } => {
                RichContentPart::from_text_or_data_url(s.clone())
            }
            NotificationValue::PluginNotification { value: other } => {
                let body = serde_json::to_string(other)
                    .unwrap_or_else(|_| String::from("<serialize error>"));
                RichContentPart::from_text_or_data_url(body)
            }
            NotificationValue::LogContent(log) => {
                RichContentPart::from(log.clone())
            }
            other => {
                let body = nv_json_without_kind(other);
                RichContentPart::from_text_or_data_url(body)
            }
        },
    }
}

/// Push one logical array element to `blocks`. `Text` parts emit a
/// single quoted text block; every other variant routes through the
/// SDK's `ContentBlock` and the bridge, flanked by `"` blocks so
/// the strip-media view of the response still parses as a JSON
/// array of strings.
fn push_part(part: RichContentPart, blocks: &mut Vec<Content>) {
    match part {
        RichContentPart::Text { text } => {
            blocks.push(quoted_text_block(&text));
        }
        other => {
            let cb: ContentBlock = other.into();
            // If the SDK forward-conversion produced a Text carrier
            // (remote ImageUrl, video, file_url, file_id, etc.), the
            // _meta markers ride along — but for the formatter's
            // JSON-array-of-strings invariant we just need the text
            // body, properly quoted. Route Text-carrier results
            // through the same quoting path as the plain Text part.
            match cb {
                ContentBlock::Text(t) => {
                    blocks.push(quoted_text_block(&t.text));
                }
                rich => {
                    blocks.push(Content::text("\""));
                    blocks.push(into_rmcp_content(rich));
                    blocks.push(Content::text("\""));
                }
            }
        }
    }
}

/// `Content::text(format!("\"{}\"", json_escape::escape_str(s)))` —
/// build a JSON-string-literal block from a raw string.
fn quoted_text_block(s: &str) -> Content {
    // `json_escape::escape_str` returns a Display-implementing
    // lazy wrapper; `format!` materializes the escaped string in
    // one pass with the surrounding quotes.
    Content::text(format!("\"{}\"", json_escape::escape_str(s)))
}

/// Serialize a `NotificationValue` to JSON, then drop its `"kind"`
/// discriminator. The discriminator is an internal-cli wire
/// convenience — MCP consumers don't need it.
fn nv_json_without_kind(nv: &NotificationValue) -> String {
    let mut v = serde_json::to_value(nv)
        .unwrap_or_else(|_| Value::String(String::from("<serialize error>")));
    if let Value::Object(map) = &mut v {
        // `serde_json::Map` is `indexmap`-backed under the SDK's
        // `preserve_order` feature, so `shift_remove` preserves
        // the remaining fields' source order.
        map.shift_remove("kind");
    }
    serde_json::to_string(&v).unwrap_or_else(|_| String::from("<serialize error>"))
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

#[cfg(test)]
mod tests {
    use super::*;
    use objectiveai_sdk::agent::completions::message::{
        File as FileBlob, ImageUrl, InputAudio, VideoUrl,
    };
    use objectiveai_sdk::cli::output::{
        Level, Notification, Ok as NotifOk, ToolLine,
    };
    use objectiveai_sdk::filesystem::logs::LogContent;
    use rmcp::model::RawContent;
    use serde_json::{Value, json};

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

    /// Concatenate the response's text-content bodies and skip
    /// every media block. The flanking `"` text blocks that bracket
    /// each media block are kept; together they form an empty
    /// string element (`""`) in the JSON-array view — which is what
    /// makes the strip-media result still parse as a valid
    /// `Vec<String>`.
    fn collect_body_strip_media(blocks: &[Content]) -> String {
        let mut s = String::new();
        for block in blocks {
            if let RawContent::Text(t) = &block.raw {
                s.push_str(&t.text);
            }
        }
        s
    }

    fn parse_array_of_strings(body: &str) -> Vec<String> {
        serde_json::from_str::<Vec<String>>(body)
            .unwrap_or_else(|e| panic!("body is not a JSON array of strings: {e}; body: {body}"))
    }

    #[test]
    fn empty_returns_sentinel() {
        let blocks = format_outputs(&[]);
        assert_eq!(blocks.len(), 1);
        match &blocks[0].raw {
            RawContent::Text(t) => assert_eq!(t.text, "<empty>"),
            other => panic!("expected text block, got {other:?}"),
        }
    }

    #[test]
    fn single_ok_strips_kind_and_quotes() {
        let outputs = vec![notif(NotificationValue::Ok(NotifOk { ok: true }))];
        let blocks = format_outputs(&outputs);
        let body = collect_body_strip_media(&blocks);
        let arr = parse_array_of_strings(&body);
        assert_eq!(arr, vec![r#"{"ok":true}"#]);
    }

    #[test]
    fn single_toolline_stdout_is_raw_line() {
        let outputs = vec![stdout_line("hello world\n")];
        let blocks = format_outputs(&outputs);
        let body = collect_body_strip_media(&blocks);
        let arr = parse_array_of_strings(&body);
        assert_eq!(arr, vec!["hello world\n"]);
    }

    #[test]
    fn single_toolline_stderr_is_bare_struct_json() {
        let outputs = vec![stderr_line("oops")];
        let blocks = format_outputs(&outputs);
        let body = collect_body_strip_media(&blocks);
        let arr = parse_array_of_strings(&body);
        assert_eq!(arr.len(), 1);
        let inner: Value = serde_json::from_str(&arr[0]).expect("inner is JSON");
        assert_eq!(inner["line"], "oops");
        assert_eq!(inner["stderr"], true);
        assert_eq!(inner.get("kind"), None);
    }

    #[test]
    fn plugin_notification_string_payload() {
        let outputs = vec![notif(NotificationValue::PluginNotification {
            value: Value::String("plain text".into()),
        })];
        let blocks = format_outputs(&outputs);
        let body = collect_body_strip_media(&blocks);
        let arr = parse_array_of_strings(&body);
        assert_eq!(arr, vec!["plain text"]);
    }

    #[test]
    fn plugin_notification_object_payload() {
        let outputs = vec![notif(NotificationValue::PluginNotification {
            value: json!({"hello":"world"}),
        })];
        let blocks = format_outputs(&outputs);
        let body = collect_body_strip_media(&blocks);
        let arr = parse_array_of_strings(&body);
        assert_eq!(arr, vec![r#"{"hello":"world"}"#]);
    }

    #[test]
    fn single_error_is_full_envelope_quoted() {
        let outputs = vec![err("nope")];
        let blocks = format_outputs(&outputs);
        let body = collect_body_strip_media(&blocks);
        let arr = parse_array_of_strings(&body);
        assert_eq!(arr.len(), 1);
        let inner: Value = serde_json::from_str(&arr[0]).expect("inner is JSON");
        assert_eq!(inner["type"], "error");
        assert_eq!(inner["message"], "nope");
    }

    #[test]
    fn multi_mixed_outputs() {
        let outputs = vec![
            stdout_line("a"),
            stderr_line("b"),
            notif(NotificationValue::Ok(NotifOk { ok: true })),
        ];
        let blocks = format_outputs(&outputs);
        let body = collect_body_strip_media(&blocks);
        let arr = parse_array_of_strings(&body);
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0], "a");
        let inner1: Value = serde_json::from_str(&arr[1]).expect("inner1 is JSON");
        assert_eq!(inner1["line"], "b");
        assert_eq!(inner1["stderr"], true);
        assert_eq!(arr[2], r#"{"ok":true}"#);
    }

    #[test]
    fn log_content_text() {
        let outputs = vec![notif(NotificationValue::LogContent(
            LogContent::Text { text: "hello".into() },
        ))];
        let blocks = format_outputs(&outputs);
        let body = collect_body_strip_media(&blocks);
        let arr = parse_array_of_strings(&body);
        assert_eq!(arr, vec!["hello"]);
    }

    #[test]
    fn log_content_json_value() {
        let outputs = vec![notif(NotificationValue::LogContent(
            LogContent::Json { content: json!({"x":1}) },
        ))];
        let blocks = format_outputs(&outputs);
        let body = collect_body_strip_media(&blocks);
        let arr = parse_array_of_strings(&body);
        assert_eq!(arr, vec![r#"{"x":1}"#]);
    }

    #[test]
    fn log_content_image_emits_media_block() {
        let outputs = vec![notif(NotificationValue::LogContent(
            LogContent::Image {
                image_url: ImageUrl {
                    url: "data:image/png;base64,iVBORw0KGgo".into(),
                    detail: None,
                },
            },
        ))];
        let blocks = format_outputs(&outputs);
        assert!(
            blocks
                .iter()
                .any(|b| matches!(b.raw, RawContent::Image(_))),
            "expected an Image content block"
        );
        let body = collect_body_strip_media(&blocks);
        let arr = parse_array_of_strings(&body);
        assert_eq!(arr, vec![""]);
    }

    #[test]
    fn log_content_audio_emits_audio_block() {
        let outputs = vec![notif(NotificationValue::LogContent(
            LogContent::Audio {
                input_audio: InputAudio {
                    data: "SUQzBAA".into(),
                    format: "audio/mpeg".into(),
                },
            },
        ))];
        let blocks = format_outputs(&outputs);
        assert!(
            blocks
                .iter()
                .any(|b| matches!(b.raw, RawContent::Audio(_))),
            "expected an Audio content block"
        );
        let body = collect_body_strip_media(&blocks);
        let arr = parse_array_of_strings(&body);
        assert_eq!(arr, vec![""]);
    }

    /// After the RichContentPart refactor, video data URLs route
    /// through the SDK's `From<RichContentPart>` impl. Since the
    /// impl never emits EmbeddedResource (per the round-trip design),
    /// videos arrive as a Text carrier whose body is the data URL —
    /// which from the strip-media perspective is a plain JSON
    /// string element. No media block is emitted; the test asserts
    /// the data URL survives as a single string in the array.
    #[test]
    fn log_content_video_lands_as_text_carrier() {
        let outputs = vec![notif(NotificationValue::LogContent(
            LogContent::Video {
                video_url: VideoUrl {
                    url: "data:video/mp4;base64,AAAA".into(),
                },
            },
        ))];
        let blocks = format_outputs(&outputs);
        let body = collect_body_strip_media(&blocks);
        let arr = parse_array_of_strings(&body);
        assert_eq!(arr, vec!["data:video/mp4;base64,AAAA"]);
    }

    /// Same story for File: SDK's From<RichContentPart> uses a Text
    /// carrier (with marker meta) for file_data, so the formatter
    /// sees a Text-shaped ContentBlock and emits a quoted string.
    #[test]
    fn log_content_file_lands_as_text_carrier() {
        let outputs = vec![notif(NotificationValue::LogContent(
            LogContent::File {
                file: FileBlob {
                    file_data: Some("JVBERi0".into()),
                    filename: Some("report.pdf".into()),
                    file_id: None,
                    file_url: None,
                },
            },
        ))];
        let blocks = format_outputs(&outputs);
        let body = collect_body_strip_media(&blocks);
        let arr = parse_array_of_strings(&body);
        assert_eq!(arr.len(), 1);
        assert!(
            arr[0].starts_with("data:application/octet-stream;base64,"),
            "expected file_data data URL, got {}",
            arr[0]
        );
    }

    #[test]
    fn mixed_text_and_media_concat_parses() {
        // [stdout "before", image, stderr "after"]
        let outputs = vec![
            stdout_line("before"),
            notif(NotificationValue::LogContent(LogContent::Image {
                image_url: ImageUrl {
                    url: "data:image/png;base64,iVBORw0KGgo".into(),
                    detail: None,
                },
            })),
            stderr_line("after"),
        ];
        let blocks = format_outputs(&outputs);
        let body = collect_body_strip_media(&blocks);
        let arr = parse_array_of_strings(&body);
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0], "before");
        assert_eq!(arr[1], ""); // media slot
        let inner: Value = serde_json::from_str(&arr[2]).expect("inner is JSON");
        assert_eq!(inner["line"], "after");
    }

    #[test]
    fn special_chars_escape_cleanly() {
        let outputs = vec![stdout_line(r#"with "quotes" and \backslash"#)];
        let blocks = format_outputs(&outputs);
        let body = collect_body_strip_media(&blocks);
        let arr = parse_array_of_strings(&body);
        assert_eq!(arr, vec![r#"with "quotes" and \backslash"#]);
    }

    #[test]
    fn agent_id_stripped_from_errors() {
        let outputs = vec![err_with_agent("nope", "agent-x")];
        let blocks = format_outputs(&outputs);
        let body = collect_body_strip_media(&blocks);
        assert!(!body.contains("agent-x"), "agent value leaked: {body}");
        let arr = parse_array_of_strings(&body);
        let inner: Value = serde_json::from_str(&arr[0]).expect("inner is JSON");
        assert_eq!(inner.get("agent_id"), None);
    }

    #[test]
    fn agent_id_stripped_from_notifications() {
        let outputs = vec![notif_with_agent(
            NotificationValue::Ok(NotifOk { ok: true }),
            "agent-y",
        )];
        let blocks = format_outputs(&outputs);
        let body = collect_body_strip_media(&blocks);
        assert!(!body.contains("agent-y"), "agent value leaked: {body}");
    }

    #[test]
    fn kind_discriminator_is_stripped() {
        let outputs = vec![notif(NotificationValue::Ok(NotifOk { ok: true }))];
        let blocks = format_outputs(&outputs);
        let body = collect_body_strip_media(&blocks);
        assert!(
            !body.contains("\\\"kind\\\""),
            "kind discriminator leaked: {body}"
        );
    }
}
