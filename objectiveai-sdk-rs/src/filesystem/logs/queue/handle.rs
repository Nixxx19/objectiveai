//! Shared per-agent-id API for the SQLite `messages` databases.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex as StdMutex};

use crate::agent::completions::message::RichContent;

use super::pending::PendingNotification;
use super::schema::{self, MessageKind};

/// Shared per-agent-id API for the SQLite `messages` databases.
///
/// `Queue` is the single owner of:
/// - per-agent rusqlite connections (`Arc<Mutex<Connection>>`),
/// - the per-agent monotonic `next_index` counter,
/// - the per-agent "request row inserted" once-flag,
/// - the per-agent path-dedup set.
///
/// Every workspace-wide read/write to a per-agent db funnels through
/// this type. Cheap to clone — internal state is `Arc`-shared across
/// clones, so the LogWriter, the cli-stream writer task, and any
/// future readers can hold their own clone without contention beyond
/// the per-agent mutex.
#[derive(Clone)]
pub struct Queue {
    inner: Arc<QueueInner>,
}

struct QueueInner {
    /// `${pipes_root}` — every agent's db lives at
    /// `{root}/<agent_id>/db.sqlite`.
    root: PathBuf,
    /// `${logs_dir}` — base for any files the queue writes
    /// (notification log files today).
    logs_dir: PathBuf,
    agents: StdMutex<HashMap<String, Arc<AgentEntry>>>,
}

struct AgentEntry {
    conn: Arc<StdMutex<rusqlite::Connection>>,
    state: StdMutex<AgentMutableState>,
}

struct AgentMutableState {
    next_index: u64,
    request_inserted: bool,
    inserted_paths: HashSet<String>,
}

impl Queue {
    pub fn new(
        root: impl Into<PathBuf>,
        logs_dir: impl Into<PathBuf>,
    ) -> Self {
        Self {
            inner: Arc::new(QueueInner {
                root: root.into(),
                logs_dir: logs_dir.into(),
                agents: StdMutex::new(HashMap::new()),
            }),
        }
    }

    /// Reserve and return the next monotonic db index for an agent.
    /// Opens + seeds the agent's entry from `MAX(index)+1` on first
    /// use.
    pub async fn reserve_index(
        &self,
        agent_id: &str,
    ) -> Result<u64, super::super::super::Error> {
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
    ) -> Result<(), super::super::super::Error> {
        let entry = self.ensure_agent(agent_id).await?;
        schema::insert_async(Arc::clone(&entry.conn), kind, path, timestamp, index).await
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
    ) -> Result<bool, super::super::super::Error> {
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
        schema::insert_async(Arc::clone(&entry.conn), kind, path, timestamp, index).await?;
        Ok(true)
    }

    /// Register a path for dedup. Returns `true` if newly inserted,
    /// `false` if already present (caller should skip the insert).
    pub async fn register_path(
        &self,
        agent_id: &str,
        path: &str,
    ) -> Result<bool, super::super::super::Error> {
        let entry = self.ensure_agent(agent_id).await?;
        let mut state = entry.state.lock().expect("agent state mutex poisoned");
        Ok(state.inserted_paths.insert(path.to_string()))
    }

    /// Write a notification log file at
    /// `agents/completions/notifications/<id>_<idx>.json`, reserve
    /// the agent's next index, and return a [`PendingNotification`]
    /// the caller queues locally for a later
    /// [`Self::insert_notification`] call.
    pub async fn write_notification(
        &self,
        agent_id: &str,
        content: &RichContent,
    ) -> Result<PendingNotification, super::super::super::Error> {
        let entry = self.ensure_agent(agent_id).await?;
        let index = {
            let mut state = entry.state.lock().expect("agent state mutex poisoned");
            let idx = state.next_index;
            state.next_index += 1;
            idx
        };
        let file = super::super::log_file::LogFile {
            route: "agents/completions/notifications".to_string(),
            id: agent_id.to_string(),
            message_index: Some(index),
            media_index: None,
            extension: "json".to_string(),
            content: serde_json::to_vec_pretty(content)
                .map_err(super::super::super::Error::Serialize)?,
            suffix: None,
        };
        let path = file.path();
        let full_path = self.inner.logs_dir.join(&path);
        if let Some(parent) = full_path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                super::super::super::Error::Write(parent.to_path_buf(), e)
            })?;
        }
        tokio::fs::write(&full_path, file.content)
            .await
            .map_err(|e| super::super::super::Error::Write(full_path, e))?;
        Ok(PendingNotification {
            agent_id: agent_id.to_string(),
            index,
            path,
            timestamp: now_secs(),
        })
    }

    /// Insert a previously-reserved notification row at its already-
    /// reserved index.
    pub async fn insert_notification(
        &self,
        notification: PendingNotification,
    ) -> Result<(), super::super::super::Error> {
        self.insert(
            &notification.agent_id,
            MessageKind::AgentCompletionNotification,
            notification.path,
            notification.timestamp,
            notification.index,
        )
        .await
    }

    /// Internal: open the agent's conn + seed `next_index` the first
    /// time this id is seen. Idempotent — losing-race callers see the
    /// winner's entry.
    async fn ensure_agent(
        &self,
        agent_id: &str,
    ) -> Result<Arc<AgentEntry>, super::super::super::Error> {
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
        // Slow path: open the conn (sync but fast — fs + DDL), seed
        // next_index via the blocking pool. If another caller wins
        // the race we drop our build and use theirs.
        let conn = schema::open(&self.inner.root.join(agent_id))?;
        let conn = Arc::new(StdMutex::new(conn));
        let max = schema::max_index_async(Arc::clone(&conn)).await?;
        let entry = Arc::new(AgentEntry {
            conn,
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
