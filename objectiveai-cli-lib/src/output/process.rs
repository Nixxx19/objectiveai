use serde::{Deserialize, Serialize};

/// Runtime lifecycle events. Replaces the stray `println!`s in
/// `objectiveai-cli/src/api/detach.rs` and `objectiveai-cli/src/update.rs`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "subkind", rename_all = "snake_case")]
pub enum Process {
    /// Emitted by the parent process during `--detach`, replacing the
    /// bare `println!("PID: {pid}")` in `api/detach.rs`.
    Detached { pid: u32 },
    /// Emitted by the auto-updater when a new release is detected.
    UpdateAvailable { version: String },
    /// Emitted by the auto-updater after a new binary has been installed.
    /// Update *failures* travel through the top-level `Error` with
    /// `fatal: false`, not here.
    UpdateInstalled { version: String },
}
