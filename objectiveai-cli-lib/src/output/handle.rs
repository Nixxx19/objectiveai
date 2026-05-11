//! Destinations for [`super::Output::emit`].
//!
//! - [`Handle::Stdout`] — this process's stdout (with fatal-error
//!   mirror to stderr).
//! - [`Handle::Stdin`] — a child process's stdin, for programmatic
//!   embedders that spawn a subprocess to consume the cli's output.
//! - [`Handle::Collect`] — an in-memory shared `Vec`, for tests or
//!   in-process embedders that want the events without a subprocess.

use std::sync::Arc;

use serde::Serialize;
use tokio::process::ChildStdin;
use tokio::sync::Mutex;

use super::Output;

/// Destination for [`super::Output::emit`].
///
/// `Arc<tokio::sync::Mutex<_>>` on the non-unit variants lets the
/// handle be cloned cheaply across the command tree's call chain and
/// guarantees concurrent emits serialize correctly. `tokio::sync::Mutex`
/// (not `std::sync::Mutex`) because we hold the guard across `.await`
/// boundaries during async writes, which would deadlock with std's.
#[derive(Clone)]
pub enum Handle {
    /// Write each line to this process's stdout; mirror fatal
    /// `Output::Error` lines to stderr.
    Stdout,
    /// Write each line to a child process's stdin.
    Stdin(Arc<Mutex<ChildStdin>>),
    /// Push each emitted `Output<T>` (reserialized as `Output<Value>`
    /// for a uniform storage type) into a shared vector.
    Collect(Arc<Mutex<Vec<Output<serde_json::Value>>>>),
}

impl Default for Handle {
    fn default() -> Self {
        Handle::Stdout
    }
}

impl Handle {
    /// Emit `output` to this destination. Panics on write failure to
    /// match `println!` semantics.
    pub async fn emit<T: Serialize>(&self, output: &Output<T>) {
        match self {
            Handle::Stdout => {
                let json = serde_json::to_string(output)
                    .expect("Output<T> serializes when T: Serialize");
                println!("{json}");
                if matches!(output, Output::Error(e) if e.fatal) {
                    eprintln!("{json}");
                }
            }
            Handle::Stdin(stdin) => {
                let json = serde_json::to_string(output)
                    .expect("Output<T> serializes when T: Serialize");
                use tokio::io::AsyncWriteExt;
                let mut guard = stdin.lock().await;
                guard
                    .write_all(json.as_bytes())
                    .await
                    .expect("emit to child stdin failed");
                guard
                    .write_all(b"\n")
                    .await
                    .expect("emit to child stdin failed");
            }
            Handle::Collect(vec) => {
                // Rebuild variant-wise as Output<Value> so storage is
                // uniform without paying a full serialize → deserialize
                // roundtrip.
                let typed = match output {
                    Output::Error(e) => Output::Error(e.clone()),
                    Output::Notification(t) => Output::Notification(
                        serde_json::to_value(t)
                            .expect("T serializes when T: Serialize"),
                    ),
                    Output::Begin => Output::Begin,
                    Output::End => Output::End,
                };
                vec.lock().await.push(typed);
            }
        }
    }
}
