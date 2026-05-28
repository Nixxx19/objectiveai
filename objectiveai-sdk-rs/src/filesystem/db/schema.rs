//! `messages` table schema + sync/async sqlite primitives. The
//! [`super::messages::Queue`] is the intended caller for the
//! primitives; nothing else in the workspace should poke at them
//! directly.

use std::sync::{Arc, Mutex};

use rusqlite::Connection;

/// Discriminant for the row's payload. Persisted as TEXT via the
/// `as_str()` mapping so on-disk dumps stay readable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MessageKind {
    AgentCompletionRequest,
    FunctionExecutionRequest,
    FunctionInventionRecursiveRequest,
    /// Notifications drained from the per-agent socket and prepended
    /// to the next tool response (or written at stream end if none).
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

    /// Parse the TEXT representation produced by [`Self::as_str`]
    /// back into a `MessageKind`. Errors with
    /// `Error::InvalidPath(format!("unknown message kind: {}", s))`
    /// on an unrecognised string — mainly a guard against
    /// out-of-sync rows from a future schema.
    pub fn from_str(s: &str) -> Result<Self, super::super::Error> {
        match s {
            "agent_completion_request" => Ok(MessageKind::AgentCompletionRequest),
            "function_execution_request" => Ok(MessageKind::FunctionExecutionRequest),
            "function_invention_recursive_request" => Ok(MessageKind::FunctionInventionRecursiveRequest),
            "agent_completion_notification" => Ok(MessageKind::AgentCompletionNotification),
            "assistant_response" => Ok(MessageKind::AssistantResponse),
            "tool_response" => Ok(MessageKind::ToolResponse),
            other => Err(super::super::Error::InvalidPath(format!(
                "unknown message kind: {other}"
            ))),
        }
    }

    /// Reconstruct the on-disk file path (relative to `logs_dir`)
    /// from a (kind, agent_id, path) row.
    ///
    /// - Request rows carry the top-level response_id in `path`; the
    ///   file lives at `{route}/request/{path}.json`.
    /// - Assistant / tool rows carry the server message index in
    ///   `path`; the file is at `agents/completions/response/messages/{agent_id}_{path}.json`.
    /// - Notification rows carry the writer-reserved index in `path`;
    ///   the file is at `agents/completions/response/notifications/{agent_id}_{path}.json`.
    pub fn file_path(&self, agent_id: &str, path: &str) -> String {
        match self {
            MessageKind::AgentCompletionRequest => {
                format!("agents/completions/request/{path}.json")
            }
            MessageKind::FunctionExecutionRequest => {
                format!("functions/executions/request/{path}.json")
            }
            MessageKind::FunctionInventionRecursiveRequest => {
                format!("functions/inventions/recursive/request/{path}.json")
            }
            MessageKind::AssistantResponse | MessageKind::ToolResponse => {
                format!("agents/completions/response/messages/{agent_id}_{path}.json")
            }
            MessageKind::AgentCompletionNotification => {
                format!("agents/completions/response/notifications/{agent_id}_{path}.json")
            }
        }
    }
}

/// A single row to be inserted into the `messages` table. Produced
/// by chunk types' `produce_message_rows()`.
#[derive(Debug, Clone)]
pub struct MessageRow {
    /// Which agent the row is about (column).
    pub agent_id: String,
    pub kind: MessageKind,
    /// The chunk-given message index (assistant/tool: `MessageChunk::index()`).
    pub index: u64,
    /// Bare-id placed in the `path` column. See [`MessageKind::file_path`]
    /// for the full filesystem path reconstruction.
    pub path: String,
    /// Unix seconds; usually the chunk's `created` field.
    pub timestamp: u64,
}

/// Create every table the shared db uses if it doesn't already
/// exist. Called from [`super::connection::connection`] on first
/// open of `db.sqlite`.
///
/// Tables:
/// - `messages` — one row per request / response / notification.
/// - `messages_queue` — per-`(caller_agent_id, spawned_agent_id)`
///   watermark of the highest `messages."index"` the caller has
///   already consumed. One row per pair; the composite PRIMARY KEY
///   doubles as the lookup index.
pub fn init_tables(conn: &Connection) -> Result<(), super::super::Error> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS messages (\
            id        INTEGER PRIMARY KEY AUTOINCREMENT, \
            agent_id  TEXT NOT NULL, \
            kind      TEXT NOT NULL, \
            path      TEXT NOT NULL, \
            timestamp INTEGER NOT NULL, \
            \"index\" INTEGER NOT NULL\
        );\
        CREATE INDEX IF NOT EXISTS messages_agent_index_idx ON messages(agent_id, \"index\");\
        CREATE INDEX IF NOT EXISTS messages_agent_idx ON messages(agent_id);\
        CREATE TABLE IF NOT EXISTS messages_queue (\
            caller_agent_id  TEXT NOT NULL, \
            spawned_agent_id TEXT NOT NULL, \
            \"index\"        INTEGER NOT NULL, \
            PRIMARY KEY (caller_agent_id, spawned_agent_id)\
        );",
    )?;
    Ok(())
}

/// `SELECT MAX("index") FROM messages WHERE agent_id = ?`. `None` when
/// no row matches.
pub fn max_index(
    conn: &Connection,
    agent_id: &str,
) -> Result<Option<u64>, super::super::Error> {
    let mut stmt = conn
        .prepare_cached("SELECT MAX(\"index\") FROM messages WHERE agent_id = ?1")?;
    use rusqlite::OptionalExtension as _;
    let row: Option<Option<i64>> = stmt
        .query_row([agent_id], |r| r.get::<_, Option<i64>>(0))
        .optional()?;
    Ok(row.flatten().map(|v| v.max(0) as u64))
}

/// Insert a single row.
pub fn insert(
    conn: &Connection,
    agent_id: &str,
    kind: MessageKind,
    path: &str,
    timestamp: u64,
    index: u64,
) -> Result<(), super::super::Error> {
    conn.execute(
        "INSERT INTO messages (agent_id, kind, path, timestamp, \"index\") VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![agent_id, kind.as_str(), path, timestamp as i64, index as i64],
    )?;
    Ok(())
}

/// Async wrapper: insert one message row on the blocking pool. Locks
/// the connection only inside the blocking body so the lock never
/// crosses an `.await`.
pub async fn insert_async(
    conn: Arc<Mutex<Connection>>,
    agent_id: String,
    kind: MessageKind,
    path: String,
    timestamp: u64,
    index: u64,
) -> Result<(), super::super::Error> {
    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().expect("filesystem db mutex poisoned");
        insert(&conn, &agent_id, kind, &path, timestamp, index)
    })
    .await
    .map_err(spawn_blocking_join_err)?
}

/// Async wrapper: `SELECT MAX("index") WHERE agent_id = ?` on the
/// blocking pool.
pub async fn max_index_async(
    conn: Arc<Mutex<Connection>>,
    agent_id: String,
) -> Result<Option<u64>, super::super::Error> {
    tokio::task::spawn_blocking(move || {
        let conn = conn.lock().expect("filesystem db mutex poisoned");
        max_index(&conn, &agent_id)
    })
    .await
    .map_err(spawn_blocking_join_err)?
}

fn spawn_blocking_join_err(e: tokio::task::JoinError) -> super::super::Error {
    super::super::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
}
