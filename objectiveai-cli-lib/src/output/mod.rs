//! Structured JSON Lines output for `objectiveai-cli`.
//!
//! Every line `objectiveai-cli` writes to stdout is one [`Output`] JSON
//! object. There are two top-level shapes, discriminated by `"type"`:
//!
//! - `error` — a failure or advisory ([`Error`]).
//! - `notification` — a typed payload `T` chosen by the consumer (the
//!   CLI defines its own notification enum and parameterizes
//!   `Output<T>` over it).
//!
//! `T` is flattened into the same JSON object as the `"type"` tag via
//! serde's internal tagging, so `T` should be a struct or an
//! internally-tagged enum.

mod error;
mod handle;
pub mod notification;

pub use error::*;
pub use handle::*;
pub use notification::*;

use serde::{Deserialize, Serialize};

/// A single line of CLI output.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Output<T> {
    Error(Error),
    Notification(T),
}

impl<T: Serialize> Output<T> {
    /// Serialize as JSON and write as a single line. If `handle` is
    /// `Some`, writes the line to the child process's stdin (locking
    /// the inner mutex). If `None`, writes to stdout — and if this is
    /// a fatal [`Error`], also mirrors the line to stderr.
    ///
    /// Panics on write failure to match `println!` semantics.
    pub async fn emit(&self, handle: &Handle) {
        let json = serde_json::to_string(self).expect("Output<T> serializes when T: Serialize");
        match handle {
            Some(stdin) => {
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
            None => {
                println!("{json}");
                if matches!(self, Output::Error(e) if e.fatal) {
                    eprintln!("{json}");
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
