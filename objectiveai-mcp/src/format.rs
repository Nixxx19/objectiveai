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
//! # `type` stripping
//!
//! Generic `NotificationValue`s flow through `nv_json_without_type`
//! before reaching `from_text_or_data_url`. The `type` field is an
//! internal-cli wire convenience — MCP consumers don't need it.
//!
//! # `agent_id` stripping
//!
//! Notifications are serialized via `n.value` (the inner
//! `NotificationValue`), which has no `agent_id` field. `ToolLine`
//! has none. `Output::Error` is explicitly cleared on a clone
//! before serialization.

use objectiveai_sdk::agent::completions::message::RichContentPart;
use objectiveai_sdk::cli::output::{
    Error, NotificationValue, Output, ToolLine, TypedNotificationValue,
};
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
            NotificationValue::Typed(TypedNotificationValue::ToolLine(tl)) if is_stderr(tl) => {
                let body =
                    serde_json::to_string(tl).unwrap_or_else(|_| String::from("<serialize error>"));
                RichContentPart::from_text_or_data_url(body)
            }
            NotificationValue::Typed(TypedNotificationValue::ToolLine(tl)) => {
                RichContentPart::from_text_or_data_url(tl.line.clone())
            }
            NotificationValue::Typed(TypedNotificationValue::PluginNotification {
                value: Value::String(s),
            }) => RichContentPart::from_text_or_data_url(s.clone()),
            NotificationValue::Typed(TypedNotificationValue::PluginNotification {
                value: other,
            }) => {
                let body = serde_json::to_string(other)
                    .unwrap_or_else(|_| String::from("<serialize error>"));
                RichContentPart::from_text_or_data_url(body)
            }
            NotificationValue::Typed(TypedNotificationValue::LogContent(log)) => {
                RichContentPart::from(log.clone())
            }
            other => {
                let body = nv_json_without_type(other);
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

/// Serialize a `NotificationValue` to JSON, then drop its `"type"`
/// discriminator. The discriminator is an internal-cli wire
/// convenience — MCP consumers don't need it.
fn nv_json_without_type(nv: &NotificationValue) -> String {
    let mut v = serde_json::to_value(nv)
        .unwrap_or_else(|_| Value::String(String::from("<serialize error>")));
    if let Value::Object(map) = &mut v {
        // `serde_json::Map` is `indexmap`-backed under the SDK's
        // `preserve_order` feature, so `shift_remove` preserves
        // the remaining fields' source order.
        map.shift_remove("type");
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
    use objectiveai_sdk::cli::output::{Level, Notification, Ok as NotifOk, ToolLine};
    use objectiveai_sdk::filesystem::logs::LogContent;
    use rmcp::model::RawContent;
    use serde_json::{Value, json};

    fn notif(value: NotificationValue) -> Output {
        Output::Notification(Notification { value })
    }

    fn notif_with_agent(value: NotificationValue, agent_id: &str) -> Output {
        Output::Notification(Notification { value })
    }

    fn err(message: &str) -> Output {
        Output::Error(Error {
            r#type: objectiveai_sdk::cli::output::ErrorType::Error,
            level: Level::Error,
            fatal: false,
            message: Value::String(message.to_string()),
            agent_id: None,
        })
    }

    fn err_with_agent(message: &str, agent_id: &str) -> Output {
        Output::Error(Error {
            r#type: objectiveai_sdk::cli::output::ErrorType::Error,
            level: Level::Error,
            fatal: false,
            message: Value::String(message.to_string()),
            agent_id: Some(agent_id.to_string()),
        })
    }

    fn stdout_line(line: &str) -> Output {
        notif(NotificationValue::Typed(TypedNotificationValue::ToolLine(
            ToolLine {
                line: line.to_string(),
                stdout: Some(true),
                stderr: None,
            },
        )))
    }

    fn stderr_line(line: &str) -> Output {
        notif(NotificationValue::Typed(TypedNotificationValue::ToolLine(
            ToolLine {
                line: line.to_string(),
                stdout: None,
                stderr: Some(true),
            },
        )))
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
        let outputs = vec![notif(NotificationValue::Typed(TypedNotificationValue::Ok(
            NotifOk { ok: true },
        )))];
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
        let outputs = vec![notif(NotificationValue::Typed(
            TypedNotificationValue::PluginNotification {
                value: Value::String("plain text".into()),
            },
        ))];
        let blocks = format_outputs(&outputs);
        let body = collect_body_strip_media(&blocks);
        let arr = parse_array_of_strings(&body);
        assert_eq!(arr, vec!["plain text"]);
    }

    #[test]
    fn plugin_notification_object_payload() {
        let outputs = vec![notif(NotificationValue::Typed(
            TypedNotificationValue::PluginNotification {
                value: json!({"hello":"world"}),
            },
        ))];
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
            notif(NotificationValue::Typed(TypedNotificationValue::Ok(
                NotifOk { ok: true },
            ))),
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
        let outputs = vec![notif(NotificationValue::Typed(
            TypedNotificationValue::LogContent(LogContent::Text {
                text: "hello".into(),
            }),
        ))];
        let blocks = format_outputs(&outputs);
        let body = collect_body_strip_media(&blocks);
        let arr = parse_array_of_strings(&body);
        assert_eq!(arr, vec!["hello"]);
    }

    #[test]
    fn log_content_json_value() {
        let outputs = vec![notif(NotificationValue::Typed(
            TypedNotificationValue::LogContent(LogContent::Json {
                content: json!({"x":1}),
            }),
        ))];
        let blocks = format_outputs(&outputs);
        let body = collect_body_strip_media(&blocks);
        let arr = parse_array_of_strings(&body);
        assert_eq!(arr, vec![r#"{"x":1}"#]);
    }

    #[test]
    fn log_content_image_emits_media_block() {
        let outputs = vec![notif(NotificationValue::Typed(
            TypedNotificationValue::LogContent(LogContent::Image {
                image_url: ImageUrl {
                    url: "data:image/png;base64,iVBORw0KGgo".into(),
                    detail: None,
                },
            }),
        ))];
        let blocks = format_outputs(&outputs);
        assert!(
            blocks.iter().any(|b| matches!(b.raw, RawContent::Image(_))),
            "expected an Image content block"
        );
        let body = collect_body_strip_media(&blocks);
        let arr = parse_array_of_strings(&body);
        assert_eq!(arr, vec![""]);
    }

    #[test]
    fn log_content_audio_emits_audio_block() {
        let outputs = vec![notif(NotificationValue::Typed(
            TypedNotificationValue::LogContent(LogContent::Audio {
                input_audio: InputAudio {
                    data: "SUQzBAA".into(),
                    format: "audio/mpeg".into(),
                },
            }),
        ))];
        let blocks = format_outputs(&outputs);
        assert!(
            blocks.iter().any(|b| matches!(b.raw, RawContent::Audio(_))),
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
        let outputs = vec![notif(NotificationValue::Typed(
            TypedNotificationValue::LogContent(LogContent::Video {
                video_url: VideoUrl {
                    url: "data:video/mp4;base64,AAAA".into(),
                },
            }),
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
        let outputs = vec![notif(NotificationValue::Typed(
            TypedNotificationValue::LogContent(LogContent::File {
                file: FileBlob {
                    file_data: Some("JVBERi0".into()),
                    filename: Some("report.pdf".into()),
                    file_id: None,
                    file_url: None,
                },
            }),
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
            notif(NotificationValue::Typed(
                TypedNotificationValue::LogContent(LogContent::Image {
                    image_url: ImageUrl {
                        url: "data:image/png;base64,iVBORw0KGgo".into(),
                        detail: None,
                    },
                }),
            )),
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

    // ───────────────────────────────────────────────────────────────
    // Mangle-and-length round-trip tests.
    //
    // Each test builds a `Vec<Output>` whose i-th entry is paired
    // with a known *expected string* at index i in the strip-media
    // JSON-array view: string-carrier outputs expect their raw
    // payload, media outputs expect `""` (the empty quoted-string
    // slot that the `"`-image-`"` flanking pattern collapses to
    // when the media block is removed). The harness round-trips
    // through `format_outputs` and asserts (a) the body parses as
    // `Vec<String>`, (b) every element matches by index, (c) the
    // expected number of `RawContent::Image` blocks survives.
    // ───────────────────────────────────────────────────────────────

    /// Tiny but valid 1×1 transparent PNG, base64-encoded as a data
    /// URL. The SDK pipeline picks this up via
    /// `RichContentPart::from_text_or_data_url` → `Image` → bridge
    /// → `rmcp::Content::Image`, so the formatter emits a real
    /// `RawContent::Image` block.
    fn valid_png_data_url() -> &'static str {
        "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNkYAAAAAYAAjCB0C8AAAAASUVORK5CYII="
    }

    fn image_output() -> Output {
        notif(NotificationValue::Typed(
            TypedNotificationValue::LogContent(LogContent::Image {
                image_url: ImageUrl {
                    url: valid_png_data_url().into(),
                    detail: None,
                },
            }),
        ))
    }

    fn plugin_notification_string(s: &str) -> Output {
        notif(NotificationValue::Typed(
            TypedNotificationValue::PluginNotification {
                value: Value::String(s.to_string()),
            },
        ))
    }

    /// Adversarial string corpus shared across the mangle tests.
    /// Returns owned `String`s so the 8 KiB length-stress entry
    /// can be built at runtime via `repeat`.
    fn tricky_corpus() -> Vec<String> {
        vec![
            "plain".to_string(),
            "with \"embedded quotes\"".to_string(),
            "with\nreal\nnewlines".to_string(),
            "with\treal\ttabs".to_string(),
            "with \\backslash\\ pairs".to_string(),
            "mixed \"quotes\" + \\esc + \nnewline + \tend".to_string(),
            // Control bytes force the `\u00XX` escape path through json_escape.
            "control \x07 bell + \x1b ESC + \x00 nul".to_string(),
            "unicode ✓ ✗ → ← 漢字 🦀".to_string(),
            // Looks like a JSON payload but is opaque to the formatter —
            // must survive as a single string element.
            "{\"json\":\"in a string\",\"nested\":[1,2,3]}".to_string(),
            "\"\"".to_string(),
            String::new(),
            "x".repeat(8 * 1024),
        ]
    }

    /// Run the round-trip pipeline and assert that the strip-media
    /// JSON-array body matches `expected` element-for-element.
    /// Also asserts that the response carries `expected_image_count`
    /// `RawContent::Image` blocks (proving media survived as media,
    /// not collapsed into text).
    fn assert_strings_survive(
        outputs: &[Output],
        expected: &[&str],
        expected_image_count: usize,
    ) {
        let blocks = format_outputs(outputs);
        let body = collect_body_strip_media(&blocks);
        let arr = parse_array_of_strings(&body);
        assert_eq!(
            arr.len(),
            expected.len(),
            "array length mismatch: got {} elements, expected {}",
            arr.len(),
            expected.len()
        );
        for (i, (got, exp)) in arr.iter().zip(expected.iter()).enumerate() {
            assert_eq!(
                got, exp,
                "mismatch at index {i}: got {got:?}, expected {exp:?}"
            );
        }
        let image_count = blocks
            .iter()
            .filter(|b| matches!(b.raw, RawContent::Image(_)))
            .count();
        assert_eq!(
            image_count, expected_image_count,
            "image block count mismatch"
        );
    }

    #[test]
    fn tricky_strings_survive_roundtrip_with_images_between() {
        let corpus = tricky_corpus();

        // For each k: stdout_line(TRICKY[k]), Image, plugin_notif_str(TRICKY[k]), Image.
        // So every string sits next to a media element on at least
        // one side, and both string-carrier paths (ToolLine /
        // PluginNotification) get exercised against the full corpus.
        let mut outputs: Vec<Output> = Vec::with_capacity(corpus.len() * 4);
        for s in &corpus {
            outputs.push(stdout_line(s));
            outputs.push(image_output());
            outputs.push(plugin_notification_string(s));
            outputs.push(image_output());
        }

        let expected: Vec<&str> = corpus
            .iter()
            .flat_map(|s| [s.as_str(), "", s.as_str(), ""])
            .collect();
        let expected_images = corpus.len() * 2;
        assert_strings_survive(&outputs, &expected, expected_images);
    }

    #[test]
    fn back_to_back_images_between_strings_survive_roundtrip() {
        // [before, Image, Image, Image, between (quotes+backslash),
        //  Image, end (newlines)]
        let outputs = vec![
            stdout_line("before"),
            image_output(),
            image_output(),
            image_output(),
            stdout_line("\"between\" \\quotes\\"),
            image_output(),
            plugin_notification_string("end\nwith\nnewlines"),
        ];
        let expected: Vec<&str> = vec![
            "before",
            "",
            "",
            "",
            "\"between\" \\quotes\\",
            "",
            "end\nwith\nnewlines",
        ];
        // Four image inputs ⇒ four image blocks in the response.
        assert_strings_survive(&outputs, &expected, 4);
    }

    #[test]
    fn extreme_length_with_dense_quotes_and_image_survives() {
        // ~50+ KiB adversarial string (quotes, backslashes, real
        // \n / \t, high-bit unicode). Big enough to push the
        // pipeline past any per-chunk fast-path assumptions.
        let unit = "\"adv\" \\seg\\ \n\t mix ✓✗ ";
        let big = unit.repeat(2200);
        assert!(big.len() > 50 * 1024, "big string too small: {}", big.len());

        let outputs = vec![
            stdout_line(&big),
            image_output(),
            stdout_line("tail"),
        ];
        let blocks = format_outputs(&outputs);
        let body = collect_body_strip_media(&blocks);
        let arr = parse_array_of_strings(&body);

        assert_eq!(arr.len(), 3, "expected 3 array elements");
        // Byte-for-byte equality for the big payload so any single-
        // byte drift surfaces as a localized panic rather than a
        // multi-KB string diff.
        assert_eq!(
            arr[0].as_bytes(),
            big.as_bytes(),
            "big string differs by {} bytes at length {}",
            arr[0]
                .as_bytes()
                .iter()
                .zip(big.as_bytes())
                .filter(|(a, b)| a != b)
                .count(),
            arr[0].len(),
        );
        assert_eq!(arr[1], "", "image slot must be empty string");
        assert_eq!(arr[2], "tail");

        let image_count = blocks
            .iter()
            .filter(|b| matches!(b.raw, RawContent::Image(_)))
            .count();
        assert_eq!(image_count, 1, "exactly one image block expected");
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
            NotificationValue::Typed(TypedNotificationValue::Ok(NotifOk { ok: true })),
            "agent-y",
        )];
        let blocks = format_outputs(&outputs);
        let body = collect_body_strip_media(&blocks);
        assert!(!body.contains("agent-y"), "agent value leaked: {body}");
    }

    #[test]
    fn kind_discriminator_is_stripped() {
        let outputs = vec![notif(NotificationValue::Typed(TypedNotificationValue::Ok(
            NotifOk { ok: true },
        )))];
        let blocks = format_outputs(&outputs);
        let body = collect_body_strip_media(&blocks);
        assert!(
            !body.contains("\\\"kind\\\""),
            "kind discriminator leaked: {body}"
        );
    }
}
