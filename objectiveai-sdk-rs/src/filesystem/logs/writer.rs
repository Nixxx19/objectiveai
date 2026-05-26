use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use futures::stream::{FuturesUnordered, StreamExt};

use crate::agent::completions::response::streaming::AgentCompletionIds;

use super::{LogFile, MessageKind, MessageRow, messages_db};

/// Function-pointer signature for `produce_message_rows()` erased
/// across chunk types. The returned iterator borrows from the chunk;
/// the `for<'a>` lifetime keeps the pointer monomorphic.
pub type ProduceRows<C> =
    for<'a> fn(&'a C) -> Box<dyn Iterator<Item = MessageRow> + Send + 'a>;

/// Writes streaming chunks to the log file structure on disk, and
/// in parallel inserts request / assistant_response / tool_response
/// rows into per-agent SQLite databases.
///
/// `C` is the chunk type. The `produce` function pointer extracts
/// [`LogFile`]s from each chunk; `produce_rows` extracts
/// [`MessageRow`]s (lazy iterator).
///
/// Maintains a buffer of previously written file contents so that
/// unchanged files are not rewritten on every chunk.
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
    /// Tracks which agents already had their request row inserted.
    db_request_inserted: HashSet<String>,
    /// Tracks which `(agent_id, index, kind)` triples have already
    /// been queued, so chunk-level duplicates (same message in a
    /// later chunk because the server re-broadcast it) don't produce
    /// duplicate rows.
    inserted_rows: HashSet<(String, u64, MessageKind)>,
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
            db_request_inserted: HashSet::new(),
            inserted_rows: HashSet::new(),
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

    /// Write a chunk to disk. Files whose content hasn't changed since the
    /// last write are skipped. All file writes plus all per-agent DB
    /// inserts run concurrently — only operations targeting the same
    /// agent's db serialise (via that agent's mutex).
    pub async fn write(&mut self, chunk: &C) -> Result<(), super::super::Error>
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
        // rows + per-message rows. Everything joins at the bottom.
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

        // SQLite ops, if wired. The chunk-borrowing iterators
        // (`agent_completion_ids()` and `produce_message_rows()`)
        // only borrow `chunk`, not `self`, so we mutate `self`
        // (ensure_conn, insert into the dedup sets) inside the loops
        // without holding a self-borrow.
        if let Some(db_root) = self.messages_db_root.clone() {
            // 1. Per-agent request rows. One per newly-seen agent;
            //    discovered via `agent_completion_ids()` which already
            //    walks the chunk tree (function executions surface
            //    every inner agent's id).
            if let (Some(kind), Some(req_path)) =
                (self.request_kind, self.request_file_path.clone())
            {
                let now = now_secs();
                for agent_id in chunk.agent_completion_ids() {
                    if self.db_request_inserted.contains(agent_id) {
                        continue;
                    }
                    let conn = self.ensure_conn(&db_root, agent_id)?;
                    self.db_request_inserted.insert(agent_id.to_string());
                    ops.push(Box::pin(messages_db::insert_request_async(
                        conn,
                        kind,
                        req_path.clone(),
                        now,
                    )));
                }
            }

            // 2. Per-message rows. Iterate the chunk's
            //    `produce_message_rows()` lazily and stream straight
            //    into the ops queue — no Vec<MessageRow>.
            if let Some(rows_fn) = self.produce_rows {
                for row in rows_fn(chunk) {
                    let key = (row.agent_id.clone(), row.index, row.kind);
                    if !self.inserted_rows.insert(key) {
                        continue;
                    }
                    let conn = self.ensure_conn(&db_root, &row.agent_id)?;
                    ops.push(Box::pin(messages_db::insert_async(
                        conn,
                        row.kind,
                        row.path,
                        row.timestamp,
                        row.index,
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

    fn ensure_conn(
        &mut self,
        db_root: &std::path::Path,
        agent_id: &str,
    ) -> Result<Arc<Mutex<rusqlite::Connection>>, super::super::Error> {
        if let Some(conn) = self.db_connections.get(agent_id) {
            return Ok(Arc::clone(conn));
        }
        let conn = messages_db::open(&db_root.join(agent_id))?;
        let conn = Arc::new(Mutex::new(conn));
        self.db_connections.insert(agent_id.to_string(), Arc::clone(&conn));
        Ok(conn)
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
