mod args;
pub use args::*;
mod body;
pub use body::*;
pub mod conduit;

/// Process exit code cli-stream uses when the per-agent socket is
/// already owned by a live listener (admission-gate loss). The
/// wrapper `objectiveai-cli` maps this exact code to
/// `Error::CliStreamSlotTaken` in
/// `objectiveai-cli/src/api/stream_subprocess.rs` (kept in sync —
/// search for the same constant value there).
pub const SLOT_TAKEN_EXIT_CODE: i32 = 42;
