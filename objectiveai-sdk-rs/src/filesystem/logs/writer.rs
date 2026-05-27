use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use futures::stream::{FuturesUnordered, StreamExt};

use crate::agent::completions::message::RichContent;
use crate::agent::completions::response::streaming::AgentCompletionIds;

use super::{LogFile, MessageKind, MessageRow, messages_db};

/// Function-pointer signature for `produce_message_rows()` erased
/// across chunk types. The returned iterator borrows from the chunk;
/// the `for<'a>` lifetime keeps the pointer monomorphic.
pub type ProduceRows<C> =
    for<'a> fn(&'a C) -> Box<dyn Iterator<Item = MessageRow> + Send + 'a>;

/// Handle to a notification whose log file has been written and whose
/// per-agent DB index has been reserved by [`LogWriter::write_notification`].
/// The cli-stream queues these locally and passes them back into
/// [`LogWriter::write`] for DB insertion when the next tool response
/// for the same agent comes in — or, at stream end, into
/// [`LogWriter::finalize`].
#[derive(Debug, Clone)]
pub struct PendingNotification {
    pub agent_id: String,
    pub index: u64,
    pub path: String,
    pub timestamp: u64,
}

/// Writes streaming chunks to the log file structure on disk, and
/// in parallel inserts request / assistant_response / tool_response /
/// agent_completion_notification rows into per-agent SQLite databases.
///
/// `C` is the chunk type. The `produce` function pointer extracts
/// [`LogFile`]s from each chunk; `produce_rows` extracts
/// [`MessageRow`]s (lazy iterator).
///
/// Maintains a buffer of previously written file contents so that
/// unchanged files are not rewritten on every chunk.
///
/// The writer does NOT own the notification queue — that lives in the
/// caller (cli-stream's writer task). Notifications enter via
/// [`Self::write_notification`] (file is written immediately, index
/// reserved), and the caller passes the resulting [`PendingNotification`]s
/// back into [`Self::write`] / [`Self::finalize`] for DB insertion.
pub struct LogWriter<C> {
    logs_dir: PathBuf,
    produce: fn(&C) -> Option<Vec<LogFile>>,
    primary_id: Option<String>,
    buffer: HashMap<String, Vec<u8>>,
    /// A pre-serialized request body waiting to be written once the
    /// response ID becomes known. Carries `(route, bytes)`. Cleared
    /// after the first chunk is written.
    pending_request: Option<(String, Vec<u8>)>,
    /// Root under which `<agent_id>/db.sqlite` lives. `None` disables
    /// per-agent SQLite writes entirely.
    messages_db_root: Option<PathBuf>,
    /// `kind` for the per-agent request row. Inserted into the db of
    /// every agent surfaced by `agent_completion_ids()`. `None` skips
    /// the request row entirely (used for factories whose request
    /// kind isn't in the WORK.md list).
    request_kind: Option<MessageKind>,
    /// Function pointer that extracts [`MessageRow`]s from a chunk
    /// lazily. `None` disables row extraction even when
    /// `messages_db_root` is set (factories wire them as a pair, but
    /// this stays optional to keep `LogWriter` usable without DB
    /// writes).
    produce_rows: Option<ProduceRows<C>>,
    /// Path of the on-disk request log file (relative to `logs_dir`).
    /// Captured once on the first chunk so it can be reused as the
    /// `path` column for every agent's request row.
    request_file_path: Option<String>,
    /// Per-agent open db connections, shared across blocking tasks.
    db_connections: HashMap<String, Arc<Mutex<rusqlite::Connection>>>,
    /// Per-agent next monotonic DB index. Seeded from `MAX(index)+1`
    /// the first time an agent's connection is opened. All inserts —
    /// requests, messages, notifications — reserve from this counter
    /// so the `index` column is a single increasing sequence across
    /// kinds (a prerequisite for "notification's index precedes the
    /// tool response's index").
    next_db_index: HashMap<String, u64>,
    /// Tracks which agents already had their request row inserted.
    db_request_inserted: HashSet<String>,
    /// Dedup by path. Chunks may re-emit the same row across multiple
    /// `write()` calls as the server amends a message; we only insert
    /// each path once per stream.
    inserted_paths: HashSet<String>,
}

impl<C> LogWriter<C> {
    pub fn new(
        logs_dir: PathBuf,
        produce: fn(&C) -> Option<Vec<LogFile>>,
    ) -> Self {
        Self {
            logs_dir,
            produce,
            primary_id: None,
            buffer: HashMap::new(),
            pending_request: None,
            messages_db_root: None,
            request_kind: None,
            produce_rows: None,
            request_file_path: None,
            db_connections: HashMap::new(),
            next_db_index: HashMap::new(),
            db_request_inserted: HashSet::new(),
            inserted_paths: HashSet::new(),
        }
    }

    /// Attach a request body that will be written alongside the first
    /// response chunk. The request is serialized eagerly, but its
    /// filename depends on the response ID which is only learned from
    /// the first chunk — so the on-disk write is deferred to that
    /// moment.
    pub fn with_request<R: serde::Serialize>(
        mut self,
        route: impl Into<String>,
        request: &R,
    ) -> Result<Self, super::super::Error> {
        let bytes = serde_json::to_vec_pretty(request)
            .map_err(super::super::Error::Serialize)?;
        self.pending_request = Some((route.into(), bytes));
        Ok(self)
    }

    /// Attach the per-agent SQLite database root + a chunk-to-rows
    /// extractor. Sets the writer up to insert a request row of
    /// `request_kind` into every agent's db (discovered via
    /// `agent_completion_ids()`), and a row per `assistant_response`
    /// / `tool_response` observed in any chunk.
    ///
    /// `messages_db_root` is the cli-stream `pipes_root`
    /// (`${config_base_dir}/pipes`). The on-disk path is
    /// `${messages_db_root}/<agent_id>/db.sqlite`.
    pub fn with_messages_db(
        mut self,
        messages_db_root: impl Into<PathBuf>,
        request_kind: Option<MessageKind>,
        produce_rows: ProduceRows<C>,
    ) -> Self {
        self.messages_db_root = Some(messages_db_root.into());
        self.request_kind = request_kind;
        self.produce_rows = Some(produce_rows);
        self
    }

    /// The ID of the primary (root) log entry.
    ///
    /// Returns `None` until at least one chunk has been written.
    pub fn primary_id(&self) -> Option<&str> {
        self.primary_id.as_deref()
    }

    /// Reserves the next per-agent DB index, writes the notification
    /// content to `agents/completions/notifications/<id>_<idx>.json`
    /// immediately, and returns a [`PendingNotification`] handle the
    /// caller queues locally. The DB insert happens later — in the
    /// next `write()` call for a tool response on the same agent, or
    /// in `finalize()` at stream end.
    ///
    /// No-op (`Err` is never returned, file is not written, the
    /// handle's `path` is empty) when the writer has no
    /// `messages_db_root` configured — but this path is only reached
    /// when the factory wired up `with_messages_db`, so in practice
    /// the configured branch always runs.
    pub async fn write_notification(
        &mut self,
        agent_id: &str,
        content: &RichContent,
    ) -> Result<PendingNotification, super::super::Error> {
        let db_root = match self.messages_db_root.clone() {
            Some(p) => p,
            None => {
                // No DB configured — nothing to reserve. Return a
                // handle with an empty path so the caller can still
                // queue it harmlessly; the eventual insert is also a
                // no-op (write/finalize gate on messages_db_root).
                return Ok(PendingNotification {
                    agent_id: agent_id.to_string(),
                    index: 0,
                    path: String::new(),
                    timestamp: now_secs(),
                });
            }
        };
        self.ensure_conn_async(&db_root, agent_id).await?;
        let index = self.reserve_index(agent_id);
        let file = LogFile {
            route: "agents/completions/notifications".to_string(),
            id: agent_id.to_string(),
            message_index: Some(index),
            media_index: None,
            extension: "json".to_string(),
            content: serde_json::to_vec_pretty(content)
                .map_err(super::super::Error::Serialize)?,
            suffix: None,
        };
        let path = file.path();
        let full_path = self.logs_dir.join(&path);
        if let Some(parent) = full_path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(|e| {
                super::super::Error::Write(parent.to_path_buf(), e)
            })?;
        }
        tokio::fs::write(&full_path, file.content)
            .await
            .map_err(|e| super::super::Error::Write(full_path, e))?;
        Ok(PendingNotification {
            agent_id: agent_id.to_string(),
            index,
            path,
            timestamp: now_secs(),
        })
    }

    /// Write a chunk to disk. Files whose content hasn't changed since the
    /// last write are skipped. All file writes plus all per-agent DB
    /// inserts (requests, messages, and any drained notifications)
    /// run concurrently — only operations targeting the same agent's
    /// db serialise (via that agent's mutex).
    ///
    /// `pending` is the caller's local notification queue. For each
    /// tool-response row encountered, every queued notification with
    /// the matching `agent_id` is removed from `pending` and its
    /// `INSERT` is pushed into the same concurrent op set (at its
    /// already-reserved index — so the notification's index precedes
    /// the tool response's reserved index). Notifications for agents
    /// not in this chunk remain in `pending` for the next call.
    pub async fn write(
        &mut self,
        chunk: &C,
        pending: &mut Vec<PendingNotification>,
    ) -> Result<(), super::super::Error>
    where
        C: AgentCompletionIds,
    {
        let mut files = match (self.produce)(chunk) {
            Some(files) => files,
            None => return Ok(()),
        };

        // First-chunk: capture primary_id, flush the pending request
        // file alongside this chunk's files, and remember the request
        // file's path for per-agent request-row inserts.
        if self.primary_id.is_none() {
            if let Some(last) = files.last() {
                self.primary_id = Some(last.id.clone());
                if let Some((route, bytes)) = self.pending_request.take() {
                    let request_file = LogFile {
                        route,
                        id: last.id.clone(),
                        message_index: None,
                        media_index: None,
                        extension: "json".to_string(),
                        content: bytes,
                        suffix: Some("request"),
                    };
                    self.request_file_path = Some(request_file.path());
                    files.push(request_file);
                }
            }
        }

        // Filter out files whose content matches the buffer.
        let changed: Vec<LogFile> = files
            .into_iter()
            .filter(|file| {
                let path = file.path();
                if self
                    .buffer
                    .get(&path)
                    .map_or(false, |prev| prev == &file.content)
                {
                    return false;
                }
                self.buffer.insert(path, file.content.clone());
                true
            })
            .collect();

        // Build the concurrent op set: file writes + per-agent request
        // rows + per-message rows + drained notification rows.
        let mut ops: FuturesUnordered<
            std::pin::Pin<
                Box<
                    dyn std::future::Future<Output = Result<(), super::super::Error>>
                        + Send,
                >,
            >,
        > = FuturesUnordered::new();

        // File writes.
        for file in changed {
            let logs_dir = self.logs_dir.clone();
            ops.push(Box::pin(async move {
                let full_path = logs_dir.join(file.path());
                if let Some(parent) = full_path.parent() {
                    tokio::fs::create_dir_all(parent).await.map_err(|e| {
                        super::super::Error::Write(parent.to_path_buf(), e)
                    })?;
                }
                tokio::fs::write(&full_path, file.content)
                    .await
                    .map_err(|e| super::super::Error::Write(full_path, e))
            }));
        }

        // SQLite ops, if wired.
        if let Some(db_root) = self.messages_db_root.clone() {
            // 1. Per-agent request rows. One per newly-seen agent.
            if let (Some(kind), Some(req_path)) =
                (self.request_kind, self.request_file_path.clone())
            {
                let now = now_secs();
                // Collect new agents to avoid borrowing `chunk` across the
                // `&mut self` method call inside the loop body.
                let new_agents: Vec<String> = chunk
                    .agent_completion_ids()
                    .filter(|aid| !self.db_request_inserted.contains(*aid))
                    .map(String::from)
                    .collect();
                for agent_id in new_agents {
                    let conn = self.ensure_conn_async(&db_root, &agent_id).await?;
                    self.db_request_inserted.insert(agent_id.clone());
                    let index = self.reserve_index(&agent_id);
                    ops.push(Box::pin(messages_db::insert_async(
                        conn,
                        kind,
                        req_path.clone(),
                        now,
                        index,
                    )));
                }
            }

            // 2. Per-message rows + per-tool-response notification drain.
            if let Some(rows_fn) = self.produce_rows {
                // Same borrow-checker pattern: collect to a small Vec.
                let rows: Vec<MessageRow> = rows_fn(chunk).collect();
                for row in rows {
                    if !self.inserted_paths.insert(row.path.clone()) {
                        continue;
                    }
                    let conn = self
                        .ensure_conn_async(&db_root, &row.agent_id)
                        .await?;

                    // Tool-response rows trigger a notification drain
                    // for that agent. Notifications insert FIRST (at
                    // their reserved indices, all earlier than the
                    // index the tool-response is about to reserve).
                    if matches!(row.kind, MessageKind::ToolResponse) {
                        let agent = row.agent_id.clone();
                        let mut i = 0;
                        while i < pending.len() {
                            if pending[i].agent_id == agent
                                && !pending[i].path.is_empty()
                            {
                                let notif = pending.remove(i);
                                let conn = Arc::clone(&conn);
                                ops.push(Box::pin(messages_db::insert_async(
                                    conn,
                                    MessageKind::AgentCompletionNotification,
                                    notif.path,
                                    notif.timestamp,
                                    notif.index,
                                )));
                            } else {
                                i += 1;
                            }
                        }
                    }

                    let index = self.reserve_index(&row.agent_id);
                    ops.push(Box::pin(messages_db::insert_async(
                        conn,
                        row.kind,
                        row.path,
                        row.timestamp,
                        index,
                    )));
                }
            }
        }

        // Drive everything to completion. First error short-circuits;
        // remaining futures are dropped as the FuturesUnordered drops.
        while let Some(result) = ops.next().await {
            result?;
        }
        Ok(())
    }

    /// Drain any remaining notifications into their respective per-
    /// agent dbs. Called by the cli-stream writer task after the chunk
    /// channel closes and any in-flight notifications have been pulled
    /// off the wire. Each surviving notification is inserted at its
    /// already-reserved index.
    pub async fn finalize(
        &mut self,
        pending: &mut Vec<PendingNotification>,
    ) -> Result<(), super::super::Error> {
        let db_root = match self.messages_db_root.clone() {
            Some(p) => p,
            None => {
                pending.clear();
                return Ok(());
            }
        };
        let mut ops: FuturesUnordered<
            std::pin::Pin<
                Box<
                    dyn std::future::Future<Output = Result<(), super::super::Error>>
                        + Send,
                >,
            >,
        > = FuturesUnordered::new();
        for notif in pending.drain(..) {
            if notif.path.is_empty() {
                continue;
            }
            let conn = self.ensure_conn_async(&db_root, &notif.agent_id).await?;
            ops.push(Box::pin(messages_db::insert_async(
                conn,
                MessageKind::AgentCompletionNotification,
                notif.path,
                notif.timestamp,
                notif.index,
            )));
        }
        while let Some(result) = ops.next().await {
            result?;
        }
        Ok(())
    }

    /// Open + cache the conn for an agent if not already cached, and
    /// seed `next_db_index` from `MAX(index) + 1` for the agent on
    /// first use.
    async fn ensure_conn_async(
        &mut self,
        db_root: &std::path::Path,
        agent_id: &str,
    ) -> Result<Arc<Mutex<rusqlite::Connection>>, super::super::Error> {
        if let Some(conn) = self.db_connections.get(agent_id) {
            return Ok(Arc::clone(conn));
        }
        let conn = messages_db::open(&db_root.join(agent_id))?;
        let conn = Arc::new(Mutex::new(conn));
        self.db_connections
            .insert(agent_id.to_string(), Arc::clone(&conn));
        // Seed next_db_index for this agent.
        let max = messages_db::max_index_async(Arc::clone(&conn)).await?;
        let next = max.map(|m| m + 1).unwrap_or(0);
        self.next_db_index.insert(agent_id.to_string(), next);
        Ok(conn)
    }

    /// Reserve and return the next DB index for `agent_id`. The
    /// per-agent counter must already be seeded (callers always
    /// invoke `ensure_conn_async` first).
    fn reserve_index(&mut self, agent_id: &str) -> u64 {
        let slot = self
            .next_db_index
            .get_mut(agent_id)
            .expect("next_db_index seeded by ensure_conn_async");
        let cur = *slot;
        *slot += 1;
        cur
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
