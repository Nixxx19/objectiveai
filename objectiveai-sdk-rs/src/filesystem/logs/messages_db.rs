//! Per-agent SQLite database alongside the agent's pipe socket.
//!
//! Each agent surfaced by a stream gets a folder at
//! `${pipes_root}/<agent_id>/`. Inside that folder sits the `socket`
//! file (bound by `objectiveai-cli-stream`'s pipe registry) and a
//! sibling `db.sqlite`. The log writer opens `db.sqlite` on first
//! chunk and inserts one row into the `messages` table for every
//! request, assistant response, and tool response observed.
//!
//! The schema is intentionally minimal — paths into the on-disk log
//! tree plus a per-call `kind` discriminant. The `index` column
//! continues monotonically across continuation rounds (a new call
//! that resumes an existing agent picks up where the prior call's
//! `MAX(index)` left off).

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use rusqlite::Connection;

/// Discriminant for the row's payload. Persisted as TEXT via the
/// `as_str()` mapping so on-disk dumps stay readable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessageKind {
    AgentCompletionRequest,
    FunctionExecutionRequest,
    FunctionInventionRecursiveRequest,
    /// Reserved — no rows are written for this kind yet.
    AgentCompletionNotification,
    AssistantResponse,
    ToolResponse,
}

impl MessageKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            MessageKind::AgentCompletionRequest => "agent_completion_request",
            MessageKind::FunctionExecutionRequest => "function_execution_request",
            MessageKind::FunctionInventionRecursiveRequest => "function_invention_recursive_request",
            MessageKind::AgentCompletionNotification => "agent_completion_notification",
            MessageKind::AssistantResponse => "assistant_response",
            MessageKind::ToolResponse => "tool_response",
        }
    }
}

/// A single row to be inserted into an agent's `messages` table.
/// Extracted from a streaming chunk by `produce_message_rows()`.
#[derive(Debug, Clone)]
pub struct MessageRow {
    /// Which agent's database receives this row.
    pub agent_id: String,
    pub kind: MessageKind,
    /// The chunk-given message index (assistant/tool: `MessageChunk::index()`).
    pub index: u64,
    /// Log file path relative to `${logs_dir}`.
    pub path: String,
    /// Unix seconds; usually the chunk's `created` field.
    pub timestamp: u64,
}

/// Open (or create) `<agent_dir>/db.sqlite`, ensuring the parent
/// directory exists and the schema is initialised. WAL-mode is
/// enabled so concurrent readers don't block the writer.
pub fn open(agent_dir: &Path) -> Result<Connection, super::super::Error> {
    std::fs::create_dir_all(agent_dir)
        .map_err(|e| super::super::Error::Write(agent_dir.to_path_buf(), e))?;
    let db_path = agent_dir.join("db.sqlite");
    let conn = Connection::open(&db_path)?;
    conn.execute_batch("PRAGMA journal_mode=WAL;")?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS messages (\
            id INTEGER PRIMARY KEY AUTOINCREMENT, \
            kind TEXT NOT NULL, \
            path TEXT NOT NULL, \
            timestamp INTEGER NOT NULL, \
            \"index\" INTEGER NOT NULL\
        );\
        CREATE INDEX IF NOT EXISTS messages_index_idx ON messages(\"index\");",
    )?;
    Ok(conn)
}

/// `SELECT MAX("index") FROM messages`. `None` when the table is empty.
pub fn max_index(conn: &Connection) -> Result<Option<u64>, super::super::Error> {
    let mut stmt = conn.prepare_cached("SELECT MAX(\"index\") FROM messages")?;
    use rusqlite::OptionalExtension as _;
    let row: Option<Option<i64>> = stmt
        .query_row([], |r| r.get::<_, Option<i64>>(0))
        .optional()?;
    Ok(row.flatten().map(|v| v.max(0) as u64))
}

/// Insert a single row.
pub fn insert(
    conn: &Connection,
    kind: MessageKind,
    path: &str,
    timestamp: u64,
    index: u64,
) -> Result<(), super::super::Error> {
    conn.execute(
        "INSERT INTO messages (kind, path, timestamp, \"index\") VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![kind.as_str(), path, timestamp as i64, index as i64],
    )?;
    Ok(())
}

/// Async wrapper: open the agent's db on the blocking pool, return
/// it wrapped in `Arc<Mutex<...>>` ready to be cloned and moved into
/// further `spawn_blocking` ops.
pub async fn open_async(
    agent_dir: PathBuf,
) -> Result<Arc<Mutex<Connection>>, super::super::Error> {
    tokio::task::spawn_blocking(move || open(&agent_dir))
        .await
        .map_err(spawn_blocking_join_err)?
        .map(|conn| Arc::new(Mutex::new(conn)))
}

/// Async wrapper: insert one message row on the blocking pool. Locks
/// the connection only inside the blocking body so the lock never
/// crosses an `.await`.
pub async fn insert_async(
    conn: Arc<Mutex<Connection>>,
    kind: MessageKind,
    path: String,
    timestamp: u64,
    index: u64,
) -> Result<(), super::super::Error> {
    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().expect("messages_db mutex poisoned");
        insert(&conn, kind, &path, timestamp, index)
    })
    .await
    .map_err(spawn_blocking_join_err)?
}

/// Async wrapper: atomically compute `MAX(index) + 1` and insert a
/// row at that index. The `MAX → INSERT` pair runs under one mutex
/// lock so concurrent writers to the same db get consistent indices.
pub async fn insert_request_async(
    conn: Arc<Mutex<Connection>>,
    kind: MessageKind,
    path: String,
    timestamp: u64,
) -> Result<(), super::super::Error> {
    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().expect("messages_db mutex poisoned");
        let next_idx = max_index(&conn)?.map(|m| m + 1).unwrap_or(0);
        insert(&conn, kind, &path, timestamp, next_idx)
    })
    .await
    .map_err(spawn_blocking_join_err)?
}

fn spawn_blocking_join_err(e: tokio::task::JoinError) -> super::super::Error {
    super::super::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
}
