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

pub use error::*;

use serde::{Deserialize, Serialize};

/// A single line of CLI output.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Output<T> {
    Error(Error),
    Notification(T),
}

#[cfg(test)]
mod tests;
