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
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "kind", rename_all = "snake_case")]
    enum DummyNotification {
        Ping { message: String },
    }

    fn roundtrip<T>(out: &Output<T>) -> serde_json::Value
    where
        T: Serialize + serde::de::DeserializeOwned,
    {
        let s = serde_json::to_string(out).unwrap();
        let back: Output<T> = serde_json::from_str(&s).unwrap();
        serde_json::to_value(&back).unwrap()
    }

    #[test]
    fn error_fatal_wire_shape() {
        let out: Output<DummyNotification> = Output::Error(Error {
            level: Level::Error,
            fatal: true,
            message: "favorite not found: foo".to_string(),
        });
        let v = roundtrip(&out);
        assert_eq!(v["type"], "error");
        assert_eq!(v["level"], "error");
        assert_eq!(v["fatal"], true);
        assert_eq!(v["message"], "favorite not found: foo");
    }

    #[test]
    fn error_non_fatal_warn_wire_shape() {
        let out: Output<DummyNotification> = Output::Error(Error {
            level: Level::Warn,
            fatal: false,
            message: "auto-update failed".to_string(),
        });
        let v = roundtrip(&out);
        assert_eq!(v["type"], "error");
        assert_eq!(v["level"], "warn");
        assert_eq!(v["fatal"], false);
    }

    #[test]
    fn notification_inner_t_flattens_alongside_type_tag() {
        let out = Output::Notification(DummyNotification::Ping {
            message: "hi".to_string(),
        });
        let v = roundtrip(&out);
        // Both the outer `type` tag and the inner T's `kind` tag live
        // at the same JSON-object level — serde internal tagging.
        assert_eq!(v["type"], "notification");
        assert_eq!(v["kind"], "ping");
        assert_eq!(v["message"], "hi");
    }
}
