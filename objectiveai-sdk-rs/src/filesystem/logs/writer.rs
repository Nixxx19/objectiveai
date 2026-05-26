use std::collections::HashMap;
use std::path::PathBuf;

use super::LogFile;

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

        // The last file is always the root — capture its id on first write
        if self.primary_id.is_none() {
            if let Some(last) = files.last() {
                self.primary_id = Some(last.id.clone());
                // Flush any pending request file alongside this first chunk.
                if let Some((route, bytes)) = self.pending_request.take() {
                    files.push(LogFile {
                        route,
                        id: last.id.clone(),
                        message_index: None,
                        media_index: None,
                        extension: "json".to_string(),
                        content: bytes,
                        suffix: Some("request"),
                    });
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

        Ok(())
    }
}
