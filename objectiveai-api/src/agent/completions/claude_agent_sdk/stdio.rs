//! Wire types for the `objectiveai-claude-agent-sdk-runner-py` stdio
//! NDJSON protocol.
//!
//! The runner is a long-lived stdio server that handles many concurrent
//! requests over a single (stdin, stdout, stderr) triple. The caller
//! tags every `run` request with a string `id`; every line emitted on
//! stdout and stderr carries the same `id` so the caller can
//! demultiplex N concurrent streams.
//!
//! ## Wire shapes
//!
//! Every line is one JSON object terminated by `\n`. By convention,
//! **`type` is the first field, `id` is the second field** on every
//! tagged line. This lets readers locate the id with a small byte
//! scan past the type discriminator and reject foreign lines without
//! fully deserializing the rest. The single exception is
//! process-level `fatal` lines (no `id`), which are emitted on stderr
//! only when the runner is exiting non-zero — those have `type` first
//! and `message` (or any other field) second.
//!
//! Stdout (per-request, both variants of [`StdioOutput`] carry `id`):
//!
//! ```text
//! {"type":"event","id":"<id>","event":<T>}
//! {"type":"end","id":"<id>","status":"ok"}
//! {"type":"end","id":"<id>","status":"cancelled"}
//! {"type":"end","id":"<id>","status":"error","error":"<msg>"}
//! ```
//!
//! Stderr (per-request `diag` carries `id`; process-fatal does not):
//!
//! ```text
//! {"type":"diag","id":"<id>","level":"info|warn|error","message":"..."}
//! {"type":"fatal","message":"..."}
//! ```
//!
//! ## Fast id-prefix check
//!
//! [`StdioOutput::try_parse`] and [`StdioError::try_parse`] take the
//! request id the caller is interested in and return
//! `Ok(Some(...))` only if the line belongs to that id (or, in the
//! [`StdioError`] case, is an untagged `fatal`). They start with a
//! cheap byte-level scan that walks past the `"type":"<v>"` pair,
//! checks whether the second key is `"id"`, and (if so) compares its
//! string value to the requested id — all without invoking
//! `serde_json` on the rest of the line. Only on a match do they
//! pay for full deserialization. Lines tagged with another caller's
//! id are dropped with `Ok(None)` after a few hundred ns of scan
//! work.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::mcp_server_config::McpServerConfig;
use super::sdk_message::SDKUserMessage;

// ---------------------------------------------------------------------------
// Stdin types (caller → runner)
// ---------------------------------------------------------------------------

/// One line written to the runner's stdin. The caller picks an `id`
/// for each `Run`; the runner echoes that `id` on every outbound
/// `event`/`end`/`diag` for the request, allowing the caller to
/// demultiplex N concurrent streams.
///
/// Borrowed-everywhere shape — we never need to clone large inputs
/// (notably the SDK user message body) just to ship a request.
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StdioInput<'a> {
    /// Start a new in-flight stream.
    Run {
        id: &'a str,
        params: RunParams<'a>,
    },
    /// Best-effort abort of one in-flight stream.
    Cancel { id: &'a str },
}

/// Wire shape of the `params` object on a `run` request. Mirrors the
/// Python runner's expected schema 1:1; the field names here must
/// match what `handle_run` reads.
#[derive(Debug, Serialize)]
pub struct RunParams<'a> {
    pub model: &'a str,
    pub message: &'a SDKUserMessage,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<objectiveai::agent::claude_agent_sdk::Effort>,

    #[serde(skip_serializing_if = "is_false")]
    pub thinking_disabled: bool,

    #[serde(skip_serializing_if = "IndexMap::is_empty")]
    pub mcp_servers: &'a IndexMap<String, McpServerConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub resume: Option<&'a str>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_agent: Option<&'a str>,

    pub rate_limit_max_retries: u64,
    pub rate_limit_max_wait_secs: u64,
}

#[allow(clippy::trivially_copy_pass_by_ref)]
fn is_false(b: &bool) -> bool {
    !*b
}

// ---------------------------------------------------------------------------
// Stdout types
// ---------------------------------------------------------------------------

/// One line emitted by the runner on stdout. Always carries an `id`.
///
/// `T` is the per-event payload type — the SDKMessage type the agent
/// completions client deserializes its own JSONL into. `T` is only
/// touched on the [`StdioOutput::Event`] variant; [`StdioOutput::End`]
/// is `T`-free.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StdioOutput<T> {
    /// One SDK message for an in-flight request.
    Event {
        id: String,
        event: T,
    },
    /// Terminal line for a request — emitted exactly once per accepted
    /// `run`. The `status` discriminator is flattened into the outer
    /// object on the wire (see [`StdioEndStatus`]).
    End {
        id: String,
        #[serde(flatten)]
        status: StdioEndStatus,
    },
}

/// The `status` discriminator on a terminal `end` line, with its
/// optional `error` payload flattened into the parent object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum StdioEndStatus {
    /// `{"status":"ok"}`
    Ok,
    /// `{"status":"cancelled"}`
    Cancelled,
    /// `{"status":"error","error":"<msg>"}`
    Error { error: String },
}

// ---------------------------------------------------------------------------
// Stderr types
// ---------------------------------------------------------------------------

/// One line emitted by the runner on stderr.
///
/// Two variants:
///
/// - [`StdioError::Diag`] is per-request and carries `id`. It's the
///   normal channel for non-fatal warnings (e.g. rate-limit retry
///   notes). The `id` is always the first field on the wire.
/// - [`StdioError::Fatal`] is process-level and carries no `id`. The
///   runner only emits one of these on its way to a non-zero exit
///   (import failure, asyncio init crash, etc.). Once `main_loop` is
///   running, no untagged stderr line is ever emitted.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StdioError {
    /// Per-request diagnostic, tagged with `id`.
    Diag {
        id: String,
        level: StdioDiagLevel,
        message: String,
    },
    /// Process-level fatal — runner is exiting non-zero. Untagged.
    Fatal { message: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StdioDiagLevel {
    Info,
    Warn,
    Error,
}

// ---------------------------------------------------------------------------
// Parse error
// ---------------------------------------------------------------------------

#[derive(Debug, Error)]
pub enum StdioParseError {
    /// Line is empty or whitespace-only.
    #[error("empty line")]
    Empty,
    /// Line doesn't start with `{` or is otherwise not a JSON object.
    #[error("malformed json envelope: expected '{{' at offset {0}")]
    NotAnObject(usize),
    /// A field name (string literal) couldn't be read at the given
    /// offset — missing quotes, immediate EOF, etc.
    #[error("malformed field name at offset {0}")]
    MalformedFieldName(usize),
    /// Reached end of line while still inside a string literal.
    #[error("unterminated string at offset {0}")]
    UnterminatedString(usize),
    /// Bytes after a field name are not `:` (with optional surrounding
    /// whitespace).
    #[error("expected ':' at offset {0}")]
    MissingColon(usize),
    /// First field name is not `type`.
    #[error("expected `type` as the first field")]
    MissingTypeField,
    /// First-field value (`type`) is not a JSON string.
    #[error("`type` field is not a string")]
    TypeNotString,
    /// Bytes between fields are not `,` (with optional surrounding
    /// whitespace).
    #[error("expected ',' between fields at offset {0}")]
    MissingComma(usize),
    /// Second field is `id` but its value is not a JSON string.
    #[error("`id` field is not a string")]
    IdNotString,
    /// Schema requires `id` as the second field but the line had none.
    #[error("missing `id` as second field")]
    MissingIdField,
    /// `serde_json::from_str` failed on a line whose id matched.
    #[error("deserialize: {0}")]
    Deserialize(#[from] serde_json::Error),
}

// ---------------------------------------------------------------------------
// Fast id-prefix scan
// ---------------------------------------------------------------------------

/// Outcome of the prefix scan.
enum IdPrefix<'a> {
    /// Second field is `id` with the given string value (raw bytes
    /// between the surrounding quotes — escapes intentionally not
    /// resolved; ids are caller-supplied and expected to be plain
    /// ASCII without escape sequences).
    Id(&'a str),
    /// First field is `type` and the JSON is well-formed past it, but
    /// the second field's name is not `id` (or the object has only one
    /// field). Used to recognize untagged `fatal` lines on stderr.
    NoId,
}

/// Skip the JSON string literal that starts at `bytes[i]` (which must
/// be the opening `"`). Returns the byte index *immediately after* the
/// closing `"`, or an error if the literal is unterminated. JSON
/// escape pairs (`\"`, `\\`, etc.) are stepped over without being
/// interpreted.
fn skip_string_literal(
    bytes: &[u8],
    mut i: usize,
) -> Result<usize, StdioParseError> {
    let len = bytes.len();
    debug_assert!(i < len && bytes[i] == b'"');
    let start = i;
    i += 1;
    while i < len && bytes[i] != b'"' {
        if bytes[i] == b'\\' && i + 1 < len {
            i += 2;
        } else {
            i += 1;
        }
    }
    if i >= len {
        return Err(StdioParseError::UnterminatedString(start + 1));
    }
    Ok(i + 1)
}

/// Skip ASCII whitespace; return the new index.
fn skip_ws(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    i
}

/// Scan `line` looking for `"type":"<v>","id":"<id>"` at the very
/// front of the JSON object (allowing only whitespace before/after
/// the `{` and around separators). The `type` value is parsed but
/// not interpreted — `serde_json` handles the discriminator on the
/// full deserialize. Does **not** unescape JSON inside the id value;
/// caller-supplied ids are constrained to plain ASCII.
fn scan_id_prefix(line: &str) -> Result<IdPrefix<'_>, StdioParseError> {
    let bytes = line.as_bytes();
    let len = bytes.len();
    let mut i = skip_ws(bytes, 0);
    if i == len {
        return Err(StdioParseError::Empty);
    }

    // Opening `{`.
    if bytes[i] != b'{' {
        return Err(StdioParseError::NotAnObject(i));
    }
    i += 1;
    i = skip_ws(bytes, i);

    // Empty object — no `type`, no `id`. Treat the same as a fatal
    // (caller's full deserialize will reject it cleanly if invalid).
    if i < len && bytes[i] == b'}' {
        return Err(StdioParseError::MissingTypeField);
    }

    // First field name — must be `"type"`.
    if i >= len || bytes[i] != b'"' {
        return Err(StdioParseError::MalformedFieldName(i));
    }
    let key_start = i + 1;
    i = skip_string_literal(bytes, i)?;
    let first_key = &line[key_start..i - 1];
    if first_key != "type" {
        return Err(StdioParseError::MissingTypeField);
    }

    // `:` after `type`.
    i = skip_ws(bytes, i);
    if i >= len || bytes[i] != b':' {
        return Err(StdioParseError::MissingColon(i));
    }
    i += 1;
    i = skip_ws(bytes, i);

    // `type`'s value must be a string. We don't care what it is — we
    // just need to step past it to reach the next field.
    if i >= len || bytes[i] != b'"' {
        return Err(StdioParseError::TypeNotString);
    }
    i = skip_string_literal(bytes, i)?;
    i = skip_ws(bytes, i);

    // After `"type":"<v>"` we expect either `,` (more fields) or `}`
    // (the object ends here — no `id`, treat as a no-id line).
    if i < len && bytes[i] == b'}' {
        return Ok(IdPrefix::NoId);
    }
    if i >= len || bytes[i] != b',' {
        return Err(StdioParseError::MissingComma(i));
    }
    i += 1;
    i = skip_ws(bytes, i);

    // Second field name. If it's `"id"`, read the id value.
    // Otherwise this is a no-id line (e.g. `fatal` whose second
    // field is `message`).
    if i >= len || bytes[i] != b'"' {
        return Err(StdioParseError::MalformedFieldName(i));
    }
    let key_start = i + 1;
    i = skip_string_literal(bytes, i)?;
    let second_key = &line[key_start..i - 1];
    if second_key != "id" {
        return Ok(IdPrefix::NoId);
    }

    // `:` after `id`.
    i = skip_ws(bytes, i);
    if i >= len || bytes[i] != b':' {
        return Err(StdioParseError::MissingColon(i));
    }
    i += 1;
    i = skip_ws(bytes, i);

    // `id` value must be a string.
    if i >= len || bytes[i] != b'"' {
        return Err(StdioParseError::IdNotString);
    }
    let val_start = i + 1;
    let after = skip_string_literal(bytes, i)?;
    Ok(IdPrefix::Id(&line[val_start..after - 1]))
}

/// Quickly recover the request `id` from a single output line without
/// running `serde_json` on the whole payload.
///
/// Returns:
///
/// - `Ok(Some(id))` — line is well-formed and its second field is
///   `id` (the per-request convention). The returned `&str` borrows
///   from `line`.
/// - `Ok(None)` — line is well-formed but has no `id` second field
///   (the untagged `fatal` carve-out).
/// - `Err(_)` — line is malformed.
///
/// The dispatcher uses this to route lines to the right per-request
/// channel before paying for a full deserialize.
pub fn extract_id(line: &str) -> Result<Option<&str>, StdioParseError> {
    match scan_id_prefix(line)? {
        IdPrefix::Id(id) => Ok(Some(id)),
        IdPrefix::NoId => Ok(None),
    }
}

// ---------------------------------------------------------------------------
// try_parse — fast id-gated deserialization
// ---------------------------------------------------------------------------

impl<T> StdioOutput<T>
where
    T: for<'de> Deserialize<'de>,
{
    /// Parse `line` IFF its `id` field (the second field, after
    /// `type`) equals `request_id`.
    ///
    /// - `Ok(Some(_))` — id matched; full deserialization succeeded.
    /// - `Ok(None)` — id mismatch; the line belongs to another request.
    ///   Caller should keep reading.
    /// - `Err(_)` — line is malformed or has no `id` second field.
    ///   On the wire every [`StdioOutput`] variant requires `id`, so
    ///   a missing-id is a hard error.
    pub fn try_parse(
        line: &str,
        request_id: &str,
    ) -> Result<Option<Self>, StdioParseError> {
        match scan_id_prefix(line)? {
            IdPrefix::Id(id) if id == request_id => {
                let parsed = serde_json::from_str(line)?;
                Ok(Some(parsed))
            }
            IdPrefix::Id(_) => Ok(None),
            IdPrefix::NoId => Err(StdioParseError::MissingIdField),
        }
    }
}

impl StdioError {
    /// Parse `line` IFF it is either:
    ///
    /// - a per-request `diag` whose `id` (second field, after `type`)
    ///   equals `request_id`, or
    /// - an untagged `fatal` (which is process-level and carries no
    ///   `id` — always relevant regardless of `request_id`).
    ///
    /// Returns `Ok(None)` for a `diag` line tagged with someone
    /// else's id. Returns `Err(_)` for a malformed line.
    pub fn try_parse(
        line: &str,
        request_id: &str,
    ) -> Result<Option<Self>, StdioParseError> {
        match scan_id_prefix(line)? {
            IdPrefix::Id(id) if id == request_id => {
                let parsed = serde_json::from_str(line)?;
                Ok(Some(parsed))
            }
            IdPrefix::Id(_) => Ok(None),
            IdPrefix::NoId => {
                // Second field is not `id` — must be the untagged
                // `fatal` carve-out (process-level).
                let parsed = serde_json::from_str(line)?;
                Ok(Some(parsed))
            }
        }
    }
}
