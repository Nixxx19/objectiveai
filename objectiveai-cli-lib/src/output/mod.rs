//! Structured JSON Lines output for `objectiveai-cli`.
//!
//! Every line `objectiveai-cli` writes to stdout is one [`Output`] JSON
//! object. There are two top-level shapes, discriminated by `"type"`:
//!
//! - `error` — a failure or non-fatal advisory, with a log level and a
//!   `fatal` flag. No machine-readable error code: the message is the
//!   message.
//! - `notification` — everything else: command results, config values,
//!   acknowledgements, runtime lifecycle events. Two-tier: the outer
//!   variant chooses a category (`kind`), the inner variant chooses a
//!   specific shape (`subkind`).
//!
//! ## No-arbitrary-JSON policy
//!
//! Output shapes are strongly typed. `serde_json::Value` appears in
//! exactly two places, both explicit escape hatches:
//!
//! 1. [`LogContent::Json`] — log files contain arbitrary API payloads
//!    that we can't constrain at this layer.
//! 2. [`Config::Jq`] — output of a user-supplied `jq` filter, which by
//!    definition can produce any JSON.
//!
//! Anywhere else, structured data is a typed Rust value.

mod ack;
mod config;
mod execution;
mod instructions;
mod list;
mod log;
mod notification;
mod process;
mod resource;
mod schema;

pub use ack::*;
pub use config::*;
pub use execution::*;
pub use instructions::*;
pub use list::*;
pub use log::*;
pub use notification::*;
pub use process::*;
pub use resource::*;
pub use schema::*;

use serde::{Deserialize, Serialize};

/// A single line of CLI output.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Output {
    Error(Error),
    Notification(Notification),
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn roundtrip(out: &Output) -> serde_json::Value {
        let s = serde_json::to_string(out).unwrap();
        let back: Output = serde_json::from_str(&s).unwrap();
        serde_json::to_value(&back).unwrap()
    }

    #[test]
    fn error_wire_shape() {
        let out = Output::Error(Error {
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
    fn non_fatal_warn_wire_shape() {
        let out = Output::Error(Error {
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
    fn ack_config_set_wire_shape() {
        let out = Output::Notification(Notification::Ack(Ack::ConfigSet {
            key: "api.mode".to_string(),
        }));
        let v = roundtrip(&out);
        assert_eq!(v["type"], "notification");
        assert_eq!(v["kind"], "ack");
        assert_eq!(v["subkind"], "config_set");
        assert_eq!(v["key"], "api.mode");
    }

    #[test]
    fn config_jq_escape_hatch_wire_shape() {
        let out = Output::Notification(Notification::Config(Config::Jq {
            results: vec![json!({"hello": "world"}), json!(42), json!(null)],
        }));
        let v = roundtrip(&out);
        assert_eq!(v["type"], "notification");
        assert_eq!(v["kind"], "config");
        assert_eq!(v["subkind"], "jq");
        assert_eq!(v["results"][0], json!({"hello": "world"}));
        assert_eq!(v["results"][1], 42);
        assert_eq!(v["results"][2], serde_json::Value::Null);
    }

    #[test]
    fn log_content_json_nested_shape() {
        let out = Output::Notification(Notification::Log(Log::Content {
            content: LogContent::Json {
                value: json!({"completion": {"id": "abc"}}),
            },
        }));
        let v = roundtrip(&out);
        assert_eq!(v["type"], "notification");
        assert_eq!(v["kind"], "log");
        assert_eq!(v["subkind"], "content");
        // LogContent uses a distinct tag name (`encoding`) to avoid
        // colliding with the outer `type`/`kind`/`subkind` discriminators.
        assert_eq!(v["content"]["encoding"], "json");
        assert_eq!(v["content"]["value"], json!({"completion": {"id": "abc"}}));
    }

    #[test]
    fn process_detached_replaces_println() {
        let out = Output::Notification(Notification::Process(Process::Detached { pid: 12345 }));
        let v = roundtrip(&out);
        assert_eq!(v["type"], "notification");
        assert_eq!(v["kind"], "process");
        assert_eq!(v["subkind"], "detached");
        assert_eq!(v["pid"], 12345);
    }

    #[test]
    fn list_pair_listing_uses_pair_items() {
        let item_json = json!({
            "name": "fav",
            "function": {"remote": "github", "owner": "o", "repository": "r", "commit": "c"},
            "profile":  {"remote": "github", "owner": "o", "repository": "r", "commit": "c"},
            "note": ""
        });
        let pair_fav: PairListItem = serde_json::from_value(item_json).unwrap();
        let out = Output::Notification(Notification::List(List::Pairs {
            source: ListSource::Favorites,
            items: vec![pair_fav],
        }));
        let v = roundtrip(&out);
        assert_eq!(v["kind"], "list");
        assert_eq!(v["subkind"], "pairs");
        assert_eq!(v["source"], "favorites");
        assert_eq!(v["items"][0]["name"], "fav");
    }
}
