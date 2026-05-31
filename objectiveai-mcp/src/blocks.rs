//! MCP `Content` block construction for `NotificationValue::LogContent`.
//!
//! The cli emits typed [`LogContent`] variants for media files (image
//! / audio / video / file) after the typed-dispatch refactor at the
//! SDK filesystem layer. This module routes each variant into a
//! native MCP [`Content`] block so MCP clients see `image` /
//! `audio` / `resource` blocks instead of a giant data-URL text dump.
//!
//! - `Image` → `Content::image(data, mime)` (data URL parsed back
//!   into raw base64 + mime).
//! - `Audio` → `Content::Audio` raw block (rmcp has no convenience
//!   constructor, but the `RawAudioContent` shape mirrors
//!   `InputAudio` exactly — `data` + `mime_type`).
//! - `Video` → `Content::resource` wrapping a
//!   `ResourceContents::BlobResourceContents` (no native MCP video
//!   block; embedded resource is the most-fidelity carrier).
//! - `File`  → `Content::resource` wrapping `BlobResourceContents`
//!   with the file's data + its filename used as the URI.
//! - `Text`  → `Content::text(text)`.
//! - `Json`  → `Content::text(pretty-json)` (no native JSON block).
//!
//! Falls back to the formatter's serialized string when the
//! collected outputs don't fit the single-LogContent-notification
//! fast path.

use objectiveai_sdk::agent::completions::message::{
    File, InputAudio, VideoUrl, parse_data_url,
};
use objectiveai_sdk::cli::output::{NotificationValue, Output};
use objectiveai_sdk::filesystem::logs::LogContent;
use rmcp::model::{
    Annotated, Content, RawAudioContent, RawContent, ResourceContents,
};

use crate::format::{OutputMode, format_outputs};

/// Build the MCP tool-response content vector from collected cli
/// outputs.
///
/// Fast path: exactly one notification carrying a typed
/// [`NotificationValue::LogContent`] → emit the corresponding native
/// MCP content block (or blocks). Every other shape falls through to
/// `[Content::text(format_outputs(...))]`, preserving the existing
/// JSON-into-text behaviour.
pub fn outputs_to_content_blocks(
    mode: OutputMode,
    outputs: &[Output],
) -> Vec<Content> {
    if outputs.len() == 1 {
        if let Output::Notification(n) = &outputs[0] {
            if let NotificationValue::LogContent(log) = &n.value {
                return log_content_to_blocks(log);
            }
        }
    }
    vec![Content::text(format_outputs(mode, outputs))]
}

/// Per-variant mapping for [`LogContent`].
fn log_content_to_blocks(log: &LogContent) -> Vec<Content> {
    match log {
        LogContent::Text { text } => vec![Content::text(text.clone())],

        LogContent::Json { content } => vec![Content::text(
            serde_json::to_string_pretty(content)
                .unwrap_or_else(|e| format!("error serializing json log content: {e}")),
        )],

        LogContent::Image { image_url } => {
            // `image_url.url` is a `data:<mime>;base64,<payload>`
            // URL produced by the SDK's read path. Pull the parts
            // back out — MCP's image block stores them separately.
            if let Some((mime, payload)) = parse_data_url(&image_url.url) {
                vec![Content::image(payload.to_string(), mime.to_string())]
            } else {
                // Should not happen for SDK-emitted ImageUrls, but
                // we don't want to drop content silently.
                vec![Content::text(image_url.url.clone())]
            }
        }

        LogContent::Audio { input_audio } => {
            vec![input_audio_to_block(input_audio)]
        }

        LogContent::Video { video_url } => vec![video_url_to_block(video_url)],

        LogContent::File { file } => vec![file_to_block(file)],
    }
}

/// `InputAudio { data, format }` → MCP `audio` content block. rmcp
/// doesn't expose a `Content::audio` constructor, so we build the
/// raw shape directly. `format` rides verbatim as the MCP
/// `mime_type` — matches the convention `From<AudioContent>` uses on
/// the SDK side.
fn input_audio_to_block(audio: &InputAudio) -> Content {
    let raw = RawContent::Audio(RawAudioContent {
        data: audio.data.clone(),
        mime_type: audio.format.clone(),
    });
    Annotated {
        raw,
        annotations: None,
    }
}

/// `VideoUrl { url: data-URL }` → MCP `resource` block carrying a
/// `BlobResourceContents` with the parsed mime + base64. MCP has no
/// native video content type, so `EmbeddedResource` is the closest
/// fidelity-preserving carrier.
fn video_url_to_block(video: &VideoUrl) -> Content {
    let (mime, payload) = match parse_data_url(&video.url) {
        Some((m, p)) => (m.to_string(), p.to_string()),
        None => {
            // No data URL — fall back to text so the URL survives.
            return Content::text(video.url.clone());
        }
    };
    Content::resource(ResourceContents::BlobResourceContents {
        uri: synthetic_video_uri(&mime),
        mime_type: Some(mime),
        blob: payload,
        meta: None,
    })
}

/// Synthetic URI used when an inline media payload has no real
/// source path (the `agents read id` flow doesn't surface the
/// original on-disk filename for the video reader). Encodes the
/// mime so consumers can route by extension if they want.
fn synthetic_video_uri(mime: &str) -> String {
    let ext = mime
        .rsplit('/')
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or("bin");
    format!("inline:///video.{ext}")
}

/// `File { file_data, filename, .. }` → MCP `resource` block
/// (BlobResourceContents). Uses the filename as the URI when
/// present; mime defaults to `application/octet-stream` since the
/// cli's `File` shape doesn't carry one (the typed `read_*_file`
/// reader discards mime after computing the filename).
fn file_to_block(file: &File) -> Content {
    let blob = match &file.file_data {
        Some(b) => b.clone(),
        None => {
            // `File` without raw data is a remote pointer; surface
            // the URL or file_id as text.
            let fallback = file
                .file_url
                .clone()
                .or_else(|| file.file_id.clone())
                .or_else(|| file.filename.clone())
                .unwrap_or_else(|| "<file>".to_string());
            return Content::text(fallback);
        }
    };
    let uri = file
        .filename
        .clone()
        .map(|n| format!("inline:///{n}"))
        .unwrap_or_else(|| "inline:///file".to_string());
    Content::resource(ResourceContents::BlobResourceContents {
        uri,
        mime_type: Some("application/octet-stream".to_string()),
        blob,
        meta: None,
    })
}
