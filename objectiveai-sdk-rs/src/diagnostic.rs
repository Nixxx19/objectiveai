//! Tiny append-only diagnostic logger. Reads `OBJECTIVEAI_DIAGNOSTIC_LOG`
//! once at first use; if unset, every event is a no-op.
//!
//! Format per line: `<micros_since_epoch>|<pid>|<exe>|<event>|<kv_pairs>\n`
//!
//! When the env var ends with `/` (or `\` on Windows), it's treated as a
//! directory and each process writes to `<dir>/<exe>-<pid>.log` — easier
//! to read post-hoc because per-process log streams stay separate.
//! Otherwise it's a shared file path opened in append mode.
//!
//! **Temporary** — used to root-cause first-chunk latency. Revert once the
//! culprit is identified.

use std::fs::{File, OpenOptions};
use std::io::Write;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

static SINK: OnceLock<Option<Mutex<File>>> = OnceLock::new();
static EXE_NAME: OnceLock<String> = OnceLock::new();

fn exe_name() -> &'static str {
    EXE_NAME.get_or_init(|| {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.file_stem().map(|s| s.to_string_lossy().into_owned()))
            .unwrap_or_else(|| "unknown".into())
    })
}

fn init_sink() -> Option<Mutex<File>> {
    let raw = std::env::var("OBJECTIVEAI_DIAGNOSTIC_LOG").ok()?;
    let trimmed = raw.trim_end_matches(|c| c == '/' || c == '\\');
    let path = if trimmed.len() < raw.len() {
        std::fs::create_dir_all(trimmed).ok()?;
        std::path::PathBuf::from(trimmed)
            .join(format!("{}-{}.log", exe_name(), std::process::id()))
    } else {
        std::path::PathBuf::from(trimmed)
    };
    let file = OpenOptions::new().create(true).append(true).open(path).ok()?;
    Some(Mutex::new(file))
}

pub fn log(event: &str, kvs: &[(&str, &str)]) {
    let sink = SINK.get_or_init(init_sink);
    let Some(file) = sink else { return };
    let micros = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros())
        .unwrap_or(0);
    let mut line = format!(
        "{micros}|{}|{}|{}|",
        std::process::id(),
        exe_name(),
        event,
    );
    for (k, v) in kvs {
        line.push_str(k);
        line.push('=');
        for c in v.chars() {
            if c == '|' || c == '\n' || c == '\r' {
                line.push(' ');
            } else {
                line.push(c);
            }
        }
        line.push(' ');
    }
    line.push('\n');
    if let Ok(mut f) = file.lock() {
        let _ = f.write_all(line.as_bytes());
        let _ = f.flush();
    }
}

/// `diag!("event_name", key1 = value, key2 = expr, ...)`. Each value is
/// formatted via `Display`. No-op when `OBJECTIVEAI_DIAGNOSTIC_LOG` is
/// unset.
#[macro_export]
macro_rules! diag {
    ($event:expr $(, $k:ident = $v:expr )* $(,)?) => {{
        $crate::diagnostic::log(
            $event,
            &[$( (stringify!($k), &format!("{}", $v)[..]) ),*],
        );
    }};
}
