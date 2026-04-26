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

mod id_prefix;
pub use id_prefix::*;
mod run_params;
pub use run_params::*;
mod scanner;
pub use scanner::*;
mod stdio_diag_level;
pub use stdio_diag_level::*;
mod stdio_end_status;
pub use stdio_end_status::*;
mod stdio_error;
pub use stdio_error::*;
mod stdio_input;
pub use stdio_input::*;
mod stdio_output;
pub use stdio_output::*;
mod stdio_parse_error;
pub use stdio_parse_error::*;
