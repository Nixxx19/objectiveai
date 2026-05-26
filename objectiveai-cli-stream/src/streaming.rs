//! Shared chunk-consumption loop every endpoint reuses.
//!
//! Drains the WS chunk stream, prints each chunk as one NDJSON line
//! on stdout (matches the existing `objectiveai-cli`'s output
//! convention), ensures a per-agent pipe exists for every
//! `agent_completion_id` the chunk references, writes each chunk
//! to a [`LogWriter`] on a separate coalescing task (so log writes
//! don't gate stream consumption), accumulates chunks into a final
//! aggregate via the caller-supplied `push` closure, and emits a
//! one-shot [`LogStreamReady`] notification with the root log id as
//! soon as the first write completes. On stream end fires every
//! pipe canceller.
//!
//! ## Order on stdout
//!
//! 1. NDJSON `Notification`s wrapping each streaming chunk, in
//!    arrival order (one per chunk).
//! 2. Exactly one `Notification { value: LogStreamReady { ... } }`,
//!    emitted by the writer task after the first chunk has been
//!    written to disk. This lands interleaved with the chunk
//!    notifications — its exact position is non-deterministic but
//!    always after at least one chunk. Same ordering the regular CLI
//!    produces.
//! 3. Pipe `Error` notifications (Warn-level, non-fatal) if any.

use std::path::PathBuf;

use futures::{Stream, StreamExt};
use objectiveai_sdk::Notifier;
use objectiveai_sdk::agent::completions::response::streaming::AgentCompletionIds;
use objectiveai_sdk::cli::output::{Handle, LogStreamReady, Notification, Output};
use objectiveai_sdk::filesystem::logs::LogWriter;
use serde::Serialize;

use crate::pipes::PipeRegistry;

/// Outcome of consuming a stream: the accumulated chunk (None when
/// the stream produced zero items) + the count of chunks consumed.
pub struct Consumed<Chunk> {
    pub aggregate: Option<Chunk>,
    pub chunk_count: usize,
}

/// Drain `stream`, emit each chunk as NDJSON to `handle`'s stdout
/// destination, manage per-agent pipes, coalesce-write to the log,
/// emit `LogStreamReady` once, accumulate. On stream end (success or
/// first error), tears down every active pipe and waits for the
/// writer task to flush its final batch.
pub async fn run_chunk_loop<S, Chunk, E, F>(
    mut stream: S,
    notifier: Notifier,
    pipes_root: PathBuf,
    log_writer: LogWriter<Chunk>,
    handle: &Handle,
    push: F,
) -> Result<Consumed<Chunk>, String>
where
    S: Stream<Item = Result<Chunk, E>> + Unpin,
    Chunk: AgentCompletionIds + Serialize + Clone + Send + Sync + 'static,
    E: std::fmt::Display,
    F: Fn(&mut Chunk, &Chunk) + Clone + Send + 'static,
{
    let registry = PipeRegistry::new();
    let mut aggregate: Option<Chunk> = None;
    let mut chunk_count: usize = 0;

    // Spawn the coalescing writer task. Main loop sends each chunk
    // via the unbounded channel; the writer task batches them up
    // (merging via `push`), writes one batch at a time, and emits
    // `LogStreamReady` once the root log id is known.
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<Chunk>();
    let writer_push = push.clone();
    let writer_handle = handle.clone();
    let writer_task = tokio::spawn(async move {
        writer_loop(rx, log_writer, writer_push, writer_handle).await
    });

    let mut stream_err: Option<String> = None;
    while let Some(item) = stream.next().await {
        match item {
            Ok(chunk) => {
                // 1. Emit the chunk to stdout as one NDJSON line.
                emit_chunk(&chunk, handle).await;

                // 2. Ensure a pipe is bound for every agent id this
                //    chunk references. `ensure_pipe` is idempotent —
                //    repeat ids are no-ops.
                for agent_id in chunk.agent_completion_ids() {
                    registry
                        .ensure_pipe(agent_id, &pipes_root, notifier.clone(), handle)
                        .await;
                }

                // 3. Hand a clone to the writer task. Best-effort —
                //    if the writer aborted on a disk error, the
                //    channel is closed and we drop the send. The
                //    writer's error surfaces on join below.
                let _ = tx.send(chunk.clone());

                // 4. Accumulate main-side.
                match aggregate.as_mut() {
                    Some(acc) => push(acc, &chunk),
                    None => aggregate = Some(chunk),
                }
                chunk_count += 1;
            }
            Err(e) => {
                stream_err = Some(format!("{e}"));
                break;
            }
        }
    }

    // Close the writer channel — the writer task drains the final
    // batch and returns.
    drop(tx);

    // Tear down every pipe. Reader tasks wake from their
    // tokio::select! and unlink the FS entry.
    registry.shutdown();

    // Collect the writer task's outcome. A JoinError means the
    // writer panicked; a writer Err means a disk write failed.
    let writer_outcome = writer_task
        .await
        .map_err(|e| format!("log writer task panicked: {e}"))?;
    if let Err(e) = writer_outcome {
        return Err(format!("log writer failed: {e}"));
    }

    if let Some(e) = stream_err {
        return Err(e);
    }
    Ok(Consumed {
        aggregate,
        chunk_count,
    })
}

/// Coalescing writer loop. Mirrors `objectiveai-cli/src/log_stream::writer_loop`.
///
/// Blocks on the next chunk, drains anything that piled up while we
/// were blocked, merges the batch into the running aggregate via
/// `push`, writes once. On the first successful write, emits a
/// one-shot `LogStreamReady` carrying the root log id.
async fn writer_loop<Chunk, F>(
    mut rx: tokio::sync::mpsc::UnboundedReceiver<Chunk>,
    mut log_writer: LogWriter<Chunk>,
    push: F,
    handle: Handle,
) -> Result<(), objectiveai_sdk::filesystem::Error>
where
    F: Fn(&mut Chunk, &Chunk),
{
    let mut agg: Option<Chunk> = None;
    let mut logged_id = false;
    while let Some(first) = rx.recv().await {
        match &mut agg {
            Some(a) => push(a, &first),
            None => agg = Some(first),
        }
        while let Ok(next) = rx.try_recv() {
            if let Some(a) = &mut agg {
                push(a, &next);
            }
        }
        if let Some(a) = &agg {
            log_writer.write(a).await?;
        }
        if !logged_id {
            if let Some(id) = log_writer.primary_id() {
                emit_log_stream_ready(id, &handle).await;
                logged_id = true;
            }
        }
    }
    Ok(())
}

async fn emit_log_stream_ready(id: &str, handle: &Handle) {
    let out = Output::<LogStreamReady>::Notification(Notification {
        agent_id: None,
        value: LogStreamReady {
            log_stream_ready: id.to_string(),
        },
    });
    out.emit(handle).await;
}

async fn emit_chunk<C: Serialize>(chunk: &C, handle: &Handle) {
    let line = match serde_json::to_string(chunk) {
        Ok(s) => s,
        Err(_) => return,
    };
    // Use the cli output Handle so destination (Stdout/Collect/...)
    // is consistent with the rest of the cli's output convention.
    // For chunks we wrap them in a Notification so they ride the
    // same NDJSON envelope every other cli output line uses.
    let value: serde_json::Value = serde_json::from_str(&line).unwrap_or(serde_json::Value::Null);
    let out = Output::<serde_json::Value>::Notification(Notification {
        agent_id: None,
        value,
    });
    out.emit(handle).await;
}
