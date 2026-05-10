//! Optional subprocess emission target.
//!
//! `Handle = None` (the default) routes [`super::Output::emit`] to
//! stdout/stderr as it always has. `Handle = Some(arc)` routes every
//! emitted JSON line into the child process's stdin instead, so a
//! programmatic caller embedding the CLI can capture the output stream
//! without shell piping.

use std::sync::Arc;

use tokio::process::ChildStdin;
use tokio::sync::Mutex;

/// Optional destination for [`super::Output::emit`].
///
/// `Arc<tokio::sync::Mutex<ChildStdin>>` so the handle can be cloned
/// cheaply across the command-tree call chain (handlers take
/// `&Handle` and clone only when capturing into an async closure).
/// The mutex must be tokio's — it's held across `.await` points
/// during the async write, which would deadlock with `std::sync::Mutex`.
pub type Handle = Option<Arc<Mutex<ChildStdin>>>;
