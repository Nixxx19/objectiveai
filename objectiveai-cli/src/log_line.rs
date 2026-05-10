//! Streaming "log file is ready" handshake between the streaming `create`
//! commands and the `--detach` parent process. The parent watches the
//! child's stdout for a [`LogStreamReady`] JSONL notification and exits
//! cleanly once it sees one.

use objectiveai_cli_lib::output::{Cleared, Items, LogContent, LogStreamReady, Output};

/// Emit the log-stream-ready notification with the given log id.
pub fn emit_log_stream_ready(id: &str) {
    Output::<LogStreamReady>::Notification(LogStreamReady {
        log_stream_ready: id.to_string(),
    })
    .emit();
}

/// Translate the upstream `LogContent` (which has no serde derives)
/// into the cli-lib wire shape and emit.
pub fn emit_log_content(content: objectiveai::filesystem::logs::LogContent) {
    let wire = match content {
        objectiveai::filesystem::logs::LogContent::Json(v) => LogContent::Json { content: v },
        objectiveai::filesystem::logs::LogContent::DataUrl(s) => LogContent::DataUrl {
            content_data_url: s,
        },
    };
    Output::<LogContent>::Notification(wire).emit();
}

/// Emit a list of log directory entries as `Items<LogListItem>`.
pub fn emit_log_list(items: Vec<objectiveai::filesystem::logs::ListItem>) {
    Output::<Items<objectiveai::filesystem::logs::ListItem>>::Notification(Items { items })
        .emit();
}

/// Emit the count of cleared log files as `Cleared`.
pub fn emit_log_clear_count(count: u64) {
    Output::<Cleared>::Notification(Cleared { cleared: count }).emit();
}

/// Returns the log id if `line` is a log-stream-ready notification.
pub fn parse_log_stream_ready(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let parsed: Output<LogStreamReady> = serde_json::from_str(trimmed).ok()?;
    match parsed {
        Output::Notification(LogStreamReady { log_stream_ready }) => Some(log_stream_ready),
        Output::Error(_) => None,
    }
}
