use serde::{Deserialize, Serialize};

/// A failure or advisory written to stdout. `fatal: true` means the
/// process is exiting with a non-zero status; `fatal: false` is a
/// non-blocking warning (e.g. auto-update failed but the requested
/// command still ran).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Error {
    pub level: Level,
    pub fatal: bool,
    pub message: String,
}

impl Error {
    /// Serialize as JSON and write to stdout as a single line. If
    /// `fatal` is `true`, also write the same line to stderr.
    pub fn emit(&self) {
        let json = serde_json::to_string(self).expect("Error always serializes");
        println!("{json}");
        if self.fatal {
            eprintln!("{json}");
        }
    }
}

/// Severity matching the conventions used by bunyan / pino / `log` crate
/// JSON encoders. `fatal` is encoded separately on [`Error`].
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}
