//! Shared per-agent-id API for the `messages` table.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex as StdMutex};

use crate::agent::completions::message::RichContent;

use super::pending::PendingNotification;
use super::schema::{self, MessageKind};

/// Per-stream handle to the shared `messages` table API. Owns:
/// - the per-agent monotonic `next_index` counter,
/// - the per-agent "request row inserted" once-flag,
/// - the per-agent path-dedup set.
///
/// All db reads/writes flow through this type. `Clone` is cheap —
/// internal state is `Arc`-shared across clones, so the LogWriter,
/// the cli-stream writer task, and any future readers can hold their
/// own clone without contention beyond the per-agent mutex.
#[derive(Clone)]
pub struct Queue {
    inner: Arc<QueueInner>,
}

struct QueueInner {
    /// Shared SQLite connection (from [`super::connection::connection`]).
    conn: Arc<StdMutex<rusqlite::Connection>>,
    /// `${logs_dir}` — base for any files the queue writes
    /// (notification log files today).
    logs_dir: PathBuf,
    agents: StdMutex<HashMap<String, Arc<AgentEntry>>>,
}

struct AgentEntry {
    state: StdMutex<AgentMutableState>,
}

struct AgentMutableState {
    next_index: u64,
    request_inserted: bool,
    inserted_paths: HashSet<String>,
}

impl Queue {
    /// Build a Queue backed by the shared SQLite connection.
    /// `logs_dir` is still needed for the notification file write.
    pub fn new(
        conn: Arc<StdMutex<rusqlite::Connection>>,
        logs_dir: impl Into<PathBuf>,
    ) -> Self {
        Self {
            inner: Arc::new(QueueInner {
                conn,
                logs_dir: logs_dir.into(),
                agents: StdMutex::new(HashMap::new()),
            }),
        }
    }

    /// Reserve and return the next monotonic db index for an agent.
    /// Seeds the agent's entry from `MAX(index) WHERE agent_id = ?`
    /// + 1 on first use.
    pub async fn reserve_index(
        &self,
        agent_id: &str,
    ) -> Result<u64, super::super::Error> {
        let entry = self.ensure_agent(agent_id).await?;
        let mut state = entry.state.lock().expect("agent state mutex poisoned");
        let idx = state.next_index;
        state.next_index += 1;
        Ok(idx)
    }

    /// Insert one row at a caller-given index.
    pub async fn insert(
        &self,
        agent_id: &str,
        kind: MessageKind,
        path: String,
        timestamp: u64,
        index: u64,
    ) -> Result<(), super::super::Error> {
        self.ensure_agent(agent_id).await?;
        schema::insert_async(
            Arc::clone(&self.inner.conn),
            agent_id.to_string(),
            kind,
            path,
            timestamp,
            index,
        )
        .await
    }

    /// Insert the per-stream request row at most once per agent.
    /// Reserves the next index under the same lock so concurrent
    /// callers can't race past the dedup check. Returns `true` if
    /// the row was inserted, `false` if a prior call already did it.
    pub async fn insert_request_once(
        &self,
        agent_id: &str,
        kind: MessageKind,
        path: String,
        timestamp: u64,
    ) -> Result<bool, super::super::Error> {
        let entry = self.ensure_agent(agent_id).await?;
        let index = {
            let mut state = entry.state.lock().expect("agent state mutex poisoned");
            if state.request_inserted {
                return Ok(false);
            }
            state.request_inserted = true;
            let idx = state.next_index;
            state.next_index += 1;
            idx
        };
        schema::insert_async(
            Arc::clone(&self.inner.conn),
            agent_id.to_string(),
            kind,
            path,
            timestamp,
            index,
        )
        .await?;
        Ok(true)
    }

    /// Register a path for dedup. Returns `true` if newly inserted,
    /// `false` if already present (caller should skip the insert).
    pub async fn register_path(
        &self,
        agent_id: &str,
        path: &str,
    ) -> Result<bool, super::super::Error> {
        let entry = self.ensure_agent(agent_id).await?;
        let mut state = entry.state.lock().expect("agent state mutex poisoned");
        Ok(state.inserted_paths.insert(path.to_string()))
    }

    /// Write a notification log file at
    /// `agents/completions/notifications/<agent_id>_<idx>.json`,
    /// reserve the agent's next index, and return a
    /// [`PendingNotification`] the caller queues locally for a later
    /// [`Self::insert_notification`] call. The row's `path` column
    /// holds just `{idx}` — the agent_id is in its own column and
    /// the route is implied by the kind.
    pub async fn write_notification(
        &self,
        agent_id: &str,
        content: &RichContent,
    ) -> Result<PendingNotification, super::super::Error> {
        let entry = self.ensure_agent(agent_id).await?;
        let index = {
            let mut state = entry.state.lock().expect("agent state mutex poisoned");
            let idx = state.next_index;
            state.next_index += 1;
            idx
        };
        // On-disk filename keeps its existing shape (so files from
        // different agents don't collide in the same directory).
        let rel_path = format!(
            "agents/completions/notifications/{agent_id}_{index}.json"
        );
        let full_path = self.inner.logs_dir.join(&rel_path);
        if let Some(parent) = full_path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                super::super::Error::Write(parent.to_path_buf(), e)
            })?;
        }
        let bytes = serde_json::to_vec_pretty(content)
            .map_err(super::super::Error::Serialize)?;
        tokio::fs::write(&full_path, bytes)
            .await
            .map_err(|e| super::super::Error::Write(full_path, e))?;
        Ok(PendingNotification {
            agent_id: agent_id.to_string(),
            index,
            // DB column stores just the bare index.
            path: format!("{index}"),
            timestamp: now_secs(),
        })
    }

    /// Insert a previously-reserved notification row at its already-
    /// reserved index.
    pub async fn insert_notification(
        &self,
        notification: PendingNotification,
    ) -> Result<(), super::super::Error> {
        self.insert(
            &notification.agent_id,
            MessageKind::AgentCompletionNotification,
            notification.path,
            notification.timestamp,
            notification.index,
        )
        .await
    }

    /// Internal: ensure the agent's mutable state is initialised.
    /// Seeds `next_index` from `MAX(index) WHERE agent_id = ?` + 1
    /// the first time this id is seen. Idempotent — losing-race
    /// callers see the winner's entry.
    async fn ensure_agent(
        &self,
        agent_id: &str,
    ) -> Result<Arc<AgentEntry>, super::super::Error> {
        // Fast path.
        {
            let guard = self
                .inner
                .agents
                .lock()
                .expect("queue agents mutex poisoned");
            if let Some(entry) = guard.get(agent_id) {
                return Ok(Arc::clone(entry));
            }
        }
        // Slow path: seed next_index via the blocking pool.
        let max = schema::max_index_async(
            Arc::clone(&self.inner.conn),
            agent_id.to_string(),
        )
        .await?;
        let entry = Arc::new(AgentEntry {
            state: StdMutex::new(AgentMutableState {
                next_index: max.map(|m| m + 1).unwrap_or(0),
                request_inserted: false,
                inserted_paths: HashSet::new(),
            }),
        });
        let mut guard = self
            .inner
            .agents
            .lock()
            .expect("queue agents mutex poisoned");
        Ok(Arc::clone(
            guard.entry(agent_id.to_string()).or_insert(entry),
        ))
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
