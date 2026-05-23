//! Temporary diagnostic logging used to inspect concurrency under
//! load. Writes one line per event to the file path supplied via
//! the `OBJECTIVEAI_DIAGNOSTIC_LOG` environment variable. When the
//! env var is unset, every `proxy_log!` invocation is a no-op (the
//! formatter never runs, no allocations).
//!
//! Format: `PROXY t=<unix_ms> tag=<event_tag> [k=v]*` — one line per
//! call, mutex-serialized so concurrent writers don't interleave.
//! The leading `PROXY` is the component prefix; sibling crates use
//! `API` and `CLI`. Sorting the merged file by the `t=` field gives
//! a chronological cross-layer timeline.
//!
//! Intentionally cheap to remove: nothing depends on this module
//! except the explicit `proxy_log!` call sites and the
//! `#[macro_use] mod debug_log;` declaration in `lib.rs`.

use std::fs::OpenOptions;
use std::io::Write;
use std::sync::{Arc, Mutex, OnceLock};

#[doc(hidden)]
pub struct LogSink {
    file: Mutex<std::fs::File>,
}

impl LogSink {
    pub fn write_line(&self, line: &str) {
        if let Ok(mut f) = self.file.lock() {
            let _ = f.write_all(line.as_bytes());
            let _ = f.write_all(b"\n");
        }
    }
}

static SINK: OnceLock<Option<Arc<LogSink>>> = OnceLock::new();

fn init_sink() -> Option<Arc<LogSink>> {
    let path = std::env::var("OBJECTIVEAI_DIAGNOSTIC_LOG").ok()?;
    if path.is_empty() {
        return None;
    }
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .ok()?;
    Some(Arc::new(LogSink {
        file: Mutex::new(file),
    }))
}

#[doc(hidden)]
pub fn sink() -> Option<&'static Arc<LogSink>> {
    SINK.get_or_init(init_sink).as_ref()
}

#[doc(hidden)]
pub fn now_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

macro_rules! proxy_log {
    ($tag:literal $(, $field:ident = $value:expr)* $(,)?) => {{
        if let Some(sink) = $crate::debug_log::sink() {
            let line = format!(
                concat!("PROXY t={} tag=", $tag, $(" ", stringify!($field), "={:?}"),*),
                $crate::debug_log::now_ms()
                $(, $value)*
            );
            sink.write_line(&line);
        }
    }};
}
