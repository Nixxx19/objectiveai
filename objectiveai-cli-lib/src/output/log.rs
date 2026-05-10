use serde::{Deserialize, Serialize};

/// Log-subsystem output.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "subkind", rename_all = "snake_case")]
pub enum Log {
    /// Emitted by `<scope> logs get` and `<scope> logs subscribe` when
    /// the requested log resolves.
    Content { content: LogContent },
    /// Emitted by `<scope> logs clear` and the global `logs clear`.
    Cleared { count: u64 },
    /// Emitted by streaming `create` commands once a log id has been
    /// allocated and the log file is available — replaces the bare
    /// `println!("Logs ID: ...")` in `objectiveai-cli/src/log_line.rs`.
    StreamReady { id: String },
    /// Emitted by `<scope> logs subscribe` if no matching log appears
    /// before the timeout.
    SubscribeTimedOut,
}

/// Contents of a log file. The upstream
/// `objectiveai::filesystem::logs::LogContent` has no serde derives,
/// so we mirror its shape here with our own wire form.
///
/// `LogContent::Json` carries the unmodified API response, which is
/// arbitrary structured JSON — one of the two intentional uses of
/// `serde_json::Value` in this module.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "encoding", rename_all = "snake_case")]
pub enum LogContent {
    Json { value: serde_json::Value },
    /// `data:{mime};base64,{payload}` string for binary log content.
    DataUrl { url: String },
}
