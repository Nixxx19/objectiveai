use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use super::{LogFile, MessageKind, MessageRow, messages_db};

/// Writes streaming chunks to the log file structure on disk.
///
/// `C` is the chunk type. The `produce` function pointer extracts
/// [`LogFile`]s from each chunk.
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
    /// `kind` for the once-per-stream request row inserted into the
    /// top-level agent's db. `None` skips the request row (used for
    /// factories whose request kind isn't in the WORK.md list).
    request_kind: Option<MessageKind>,
    /// Function pointer that extracts [`MessageRow`]s from a chunk.
    /// `None` disables row extraction even when `messages_db_root` is
    /// set (factories always wire them as a pair, but this stays
    /// optional to keep `LogWriter` usable without DB writes).
    produce_rows: Option<fn(&C) -> Vec<MessageRow>>,
    /// Per-agent open db connections; lazily populated on first use.
    db_connections: HashMap<String, rusqlite::Connection>,
    /// Tracks which `(agent_id, index, kind)` triples have already
    /// been inserted, so chunk-level duplicates (same message in a
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
            db_connections: HashMap::new(),
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
    /// extractor. Sets the writer up to insert a row per request
    /// (kind given here) into the top-level agent's db on the first
    /// chunk, and a row per `assistant_response` / `tool_response`
    /// observed in any chunk.
    ///
    /// `messages_db_root` is the cli-stream `pipes_root`
    /// (`${config_base_dir}/pipes`). The on-disk path is
    /// `${messages_db_root}/<agent_id>/db.sqlite`.
    pub fn with_messages_db(
        mut self,
        messages_db_root: impl Into<PathBuf>,
        request_kind: Option<MessageKind>,
        produce_rows: fn(&C) -> Vec<MessageRow>,
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
    /// last write are skipped.
    pub async fn write(&mut self, chunk: &C) -> Result<(), super::super::Error> {
        let mut files = match (self.produce)(chunk) {
            Some(files) => files,
            None => return Ok(()),
        };

        // The last file is always the root — capture its id on first write.
        // This also flushes the pending request file (alongside the first
        // chunk's files) and, when the messages-db is wired, inserts the
        // once-per-stream request row.
        let mut first_chunk_request_path: Option<String> = None;
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
                    first_chunk_request_path = Some(request_file.path());
                    files.push(request_file);
                }
            }
        }

        // Filter out files whose content matches the buffer
        let changed: Vec<LogFile> = files.into_iter().filter(|file| {
            let path = file.path();
            if self.buffer.get(&path).map_or(false, |prev| prev == &file.content) {
                return false;
            }
            self.buffer.insert(path, file.content.clone());
            true
        }).collect();

        futures::future::try_join_all(changed.into_iter().map(|file| {
            let full_path = self.logs_dir.join(file.path());
            async move {
                if let Some(parent) = full_path.parent() {
                    tokio::fs::create_dir_all(parent).await
                        .map_err(|e| super::super::Error::Write(parent.to_path_buf(), e))?;
                }
                tokio::fs::write(&full_path, file.content).await
                    .map_err(|e| super::super::Error::Write(full_path, e))
            }
        })).await?;

        // SQLite writes: one row per request (once) + rows per assistant /
        // tool message (deduped). All inline-sync — matches `config/db.rs`.
        if let Some(db_root) = self.messages_db_root.clone() {
            // 1. Once-per-stream request row.
            if let (Some(req_path), Some(req_kind), Some(primary)) = (
                first_chunk_request_path,
                self.request_kind,
                self.primary_id.clone(),
            ) {
                let conn = self.open_or_get_conn(&db_root, &primary)?;
                let next_idx = messages_db::max_index(conn)?.map(|m| m + 1).unwrap_or(0);
                messages_db::insert(conn, req_kind, &req_path, now_secs(), next_idx)?;
            }

            // 2. Per-message rows from this chunk.
            if let Some(rows_fn) = self.produce_rows {
                let rows = rows_fn(chunk);
                for row in rows {
                    let key = (row.agent_id.clone(), row.index, row.kind);
                    if !self.inserted_rows.insert(key) {
                        continue;
                    }
                    let conn = self.open_or_get_conn(&db_root, &row.agent_id)?;
                    messages_db::insert(conn, row.kind, &row.path, row.timestamp, row.index)?;
                }
            }
        }

        Ok(())
    }

    fn open_or_get_conn(
        &mut self,
        db_root: &std::path::Path,
        agent_id: &str,
    ) -> Result<&rusqlite::Connection, super::super::Error> {
        if !self.db_connections.contains_key(agent_id) {
            let agent_dir = db_root.join(agent_id);
            let conn = messages_db::open(&agent_dir)?;
            self.db_connections.insert(agent_id.to_string(), conn);
        }
        Ok(self.db_connections.get(agent_id).unwrap())
    }
}

fn now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
