use thiserror::Error;

/// Local errors raised while parsing wire data or dealing with the codex
/// subprocess. Mirrors the error families in the Python SDK (`errors.py`),
/// minus the install/auth/abort variants which are not relevant to a
/// consumer that doesn't manage installation or login.
#[derive(Debug, Error)]
pub enum Error {
    /// A JSONL line could not be parsed into a [`super::ThreadEvent`].
    #[error("failed to parse thread event line: {0}")]
    EventParse(#[from] serde_json::Error),

    /// The codex subprocess exited with a non-zero status. The string is the
    /// captured stderr.
    #[error("codex exec failed: {0}")]
    Exec(String),

    /// The runner emitted a `turn.failed` event; the inner string is the
    /// `error.message` payload.
    #[error("thread run error: {0}")]
    ThreadRun(String),
}
