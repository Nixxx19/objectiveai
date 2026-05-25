//! Per-agent named-pipe notify bridge.
//!
//! When the WS stream surfaces a new agent completion `response_id`,
//! we bind a named pipe (Windows) / Unix domain socket (POSIX) for it
//! under `${config_base_dir}/pipes/<agent_id>`. External processes
//! that want to push a notification at that agent connect to the
//! pipe and write NDJSON lines, one [`RichContent`] per line.
//! The reader task wraps each into an [`AgentCompletionNotifyParams`]
//! (with `response_id` set to the pipe's agent id) and dispatches
//! through the shared [`Notifier`].
//!
//! ## Path semantics
//!
//! - **POSIX**: full filesystem path. Slashes in the agent id stay
//!   as slashes — they become real subdirectories under
//!   `${config_base_dir}/pipes/`. The final segment is the UDS file
//!   name. Parent directories are auto-created.
//! - **Windows**: named pipes live in the flat `\\.\pipe\` namespace
//!   and can't be hierarchical, so slashes in the agent id are
//!   re-encoded as `_` and the whole encoded value becomes the
//!   pipe name (`\\.\pipe\objectiveai_<encoded-agent-id>`).
//!
//! ## Lifecycle
//!
//! [`PipeRegistry`] holds one cancel oneshot per active agent id.
//! [`ensure_pipe`] is idempotent — calling it again for an already-
//! tracked id is a no-op. [`PipeRegistry::shutdown`] fires every
//! cancel sender; each reader task drops its listener (which unlinks
//! the FS entry on POSIX) and returns.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use dashmap::DashMap;
use interprocess::local_socket::tokio::{Listener, prelude::*};
#[cfg(unix)]
use interprocess::local_socket::ToFsName;
#[cfg(windows)]
use interprocess::local_socket::ToNsName;
use interprocess::local_socket::{ListenerOptions, Name};
use objectiveai_sdk::Notifier;
use objectiveai_sdk::agent::completions::message::RichContent;
use objectiveai_sdk::agent::completions::request::AgentCompletionNotifyParams;
use objectiveai_sdk::cli::output::{Error, Handle, Level, Output};
use tokio::io::AsyncBufReadExt;
use tokio::sync::oneshot;

/// Compute the pipe address for `agent_id` under `pipes_root`.
///
/// `pipes_root` is `${config_base_dir}/pipes`. The returned
/// [`Name`] uses a filesystem path on POSIX and a namespaced
/// name on Windows (see module-level docs).
///
/// On POSIX, also returns the parent directory + filesystem path
/// so the caller can pre-create dirs and best-effort unlink stale
/// socket files left behind by a previous abnormal exit.
pub struct PipeAddress {
    pub name: Name<'static>,
    /// `Some(path)` on POSIX (filesystem-addressed); `None` on Windows.
    pub fs_path: Option<PathBuf>,
}

pub fn pipe_address_for_agent(
    pipes_root: &Path,
    agent_id: &str,
) -> Result<PipeAddress, String> {
    #[cfg(unix)]
    {
        let fs_path = pipes_root.join(agent_id);
        let name = fs_path
            .clone()
            .to_fs_name::<interprocess::local_socket::GenericFilePath>()
            .map_err(|e| format!("invalid pipe path for agent {agent_id:?}: {e}"))?
            .into_owned();
        Ok(PipeAddress {
            name,
            fs_path: Some(fs_path),
        })
    }
    #[cfg(windows)]
    {
        let _ = pipes_root; // not used on Windows — pipes are in a flat namespace
        let encoded = agent_id.replace(['/', '\\', ':'], "_");
        let ns_name = format!("objectiveai_{encoded}");
        let name = ns_name
            .to_ns_name::<interprocess::local_socket::GenericNamespaced>()
            .map_err(|e| format!("invalid pipe name for agent {agent_id:?}: {e}"))?
            .into_owned();
        Ok(PipeAddress {
            name,
            fs_path: None,
        })
    }
}

/// Tracks active per-agent pipe listener tasks. Clone-cheap.
#[derive(Default, Clone)]
pub struct PipeRegistry {
    inner: Arc<Inner>,
}

#[derive(Default)]
struct Inner {
    cancellers: DashMap<String, oneshot::Sender<()>>,
}

impl PipeRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Bind a pipe for `agent_id` and spawn its reader task. No-op
    /// if a pipe for this id is already tracked. Errors during bind
    /// surface via `handle` and prevent the id from being inserted,
    /// so a later `ensure_pipe` call will retry.
    pub async fn ensure_pipe(
        &self,
        agent_id: &str,
        pipes_root: &Path,
        notifier: Notifier,
        handle: &Handle,
    ) {
        if self.inner.cancellers.contains_key(agent_id) {
            return;
        }

        let address = match pipe_address_for_agent(pipes_root, agent_id) {
            Ok(a) => a,
            Err(e) => {
                emit_error(handle, format!("pipe addr for {agent_id:?}: {e}")).await;
                return;
            }
        };

        #[cfg(unix)]
        if let Some(fs_path) = &address.fs_path {
            if let Some(parent) = fs_path.parent() {
                if let Err(e) = tokio::fs::create_dir_all(parent).await {
                    emit_error(
                        handle,
                        format!("mkdir parent for {}: {e}", fs_path.display()),
                    )
                    .await;
                    return;
                }
            }
            // Best-effort unlink — recover from a stale socket left
            // behind by a previous `kill -9`. `EEXIST` and the like
            // are silently swallowed; the real failure surfaces from
            // the bind below.
            let _ = tokio::fs::remove_file(fs_path).await;
        }

        let listener = match ListenerOptions::new().name(address.name).create_tokio() {
            Ok(l) => l,
            Err(e) => {
                emit_error(
                    handle,
                    format!(
                        "bind pipe for {agent_id:?}{}: {e}",
                        address
                            .fs_path
                            .as_ref()
                            .map(|p| format!(" at {}", p.display()))
                            .unwrap_or_default()
                    ),
                )
                .await;
                return;
            }
        };

        let (cancel_tx, cancel_rx) = oneshot::channel();
        let inserted = self
            .inner
            .cancellers
            .insert(agent_id.to_string(), cancel_tx);
        debug_assert!(inserted.is_none(), "ensure_pipe race: id already present");

        let task_agent_id = agent_id.to_string();
        let task_notifier = notifier;
        let task_handle = handle.clone();
        tokio::spawn(async move {
            run_listener(listener, task_agent_id, task_notifier, task_handle, cancel_rx).await;
        });
    }

    /// Fire every cancel and drop the registry. Reader tasks wake
    /// from their `tokio::select!`, drop their listeners (which
    /// unlinks the FS entry on POSIX), and return.
    pub fn shutdown(&self) {
        // Drain into a Vec so we don't hold dashmap shard locks while
        // sending on the oneshots.
        let mut all: Vec<(String, oneshot::Sender<()>)> = Vec::new();
        let keys: Vec<String> = self
            .inner
            .cancellers
            .iter()
            .map(|kv| kv.key().clone())
            .collect();
        for k in keys {
            if let Some((id, tx)) = self.inner.cancellers.remove(&k) {
                all.push((id, tx));
            }
        }
        for (_id, tx) in all {
            let _ = tx.send(());
        }
    }
}

async fn run_listener(
    listener: Listener,
    agent_id: String,
    notifier: Notifier,
    handle: Handle,
    mut cancel: oneshot::Receiver<()>,
) {
    loop {
        tokio::select! {
            _ = &mut cancel => break,
            accept = listener.accept() => {
                match accept {
                    Ok(conn) => {
                        let notifier = notifier.clone();
                        let agent_id = agent_id.clone();
                        let handle = handle.clone();
                        tokio::spawn(handle_connection(conn, agent_id, notifier, handle));
                    }
                    Err(e) => {
                        emit_error(
                            &handle,
                            format!("pipe accept for {agent_id:?}: {e}"),
                        )
                        .await;
                        // Brief backoff so a hard-broken listener
                        // doesn't spin.
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    }
                }
            }
        }
    }
}

async fn handle_connection(
    conn: interprocess::local_socket::tokio::Stream,
    agent_id: String,
    notifier: Notifier,
    handle: Handle,
) {
    let reader = tokio::io::BufReader::new(conn);
    let mut lines = reader.lines();
    loop {
        let line = match lines.next_line().await {
            Ok(Some(l)) => l,
            Ok(None) => return,
            Err(e) => {
                emit_error(
                    &handle,
                    format!("pipe read for {agent_id:?}: {e}"),
                )
                .await;
                return;
            }
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let content: RichContent = match serde_json::from_str(trimmed) {
            Ok(c) => c,
            Err(e) => {
                emit_error(
                    &handle,
                    format!(
                        "pipe line for {agent_id:?} is not a valid RichContent JSON: {e}; line: {}",
                        truncate(trimmed, 200)
                    ),
                )
                .await;
                continue;
            }
        };
        let params = AgentCompletionNotifyParams {
            response_id: agent_id.clone(),
            content,
        };
        if let Err(e) = notifier.notify(params).await {
            emit_error(
                &handle,
                format!("notify dispatch for {agent_id:?}: {e}"),
            )
            .await;
        }
    }
}

async fn emit_error(handle: &Handle, message: String) {
    let out = Output::<serde_json::Value>::Error(Error {
        level: Level::Warn,
        fatal: false,
        message: serde_json::Value::String(message),
        agent_id: None,
    });
    out.emit(handle).await;
}

fn truncate(s: &str, max_chars: usize) -> String {
    s.chars().take(max_chars).collect()
}
