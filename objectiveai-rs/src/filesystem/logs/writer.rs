use std::collections::HashMap;
use std::path::PathBuf;

use super::LogsError;

/// Writes streaming chunks to the log file structure on disk.
///
/// `C` is the chunk type. The `produce` function pointer extracts
/// `(path, bytes)` pairs from each chunk.
///
/// Maintains a buffer of previously written file contents so that
/// unchanged files are not rewritten on every chunk.
pub struct LogWriter<C> {
    logs_dir: PathBuf,
    produce: fn(&C) -> Option<Vec<(String, Vec<u8>)>>,
    primary_path: Option<String>,
    buffer: HashMap<String, Vec<u8>>,
}

impl<C> LogWriter<C> {
    pub fn new(
        logs_dir: PathBuf,
        produce: fn(&C) -> Option<Vec<(String, Vec<u8>)>>,
    ) -> Self {
        Self {
            logs_dir,
            produce,
            primary_path: None,
            buffer: HashMap::new(),
        }
    }

    /// The path of the primary (root) log file, relative to the logs directory.
    ///
    /// Returns `None` until at least one chunk has been written.
    pub fn primary_path(&self) -> Option<&str> {
        self.primary_path.as_deref()
    }

    /// Write a chunk to disk. Files whose content hasn't changed since the
    /// last write are skipped.
    pub async fn write(&mut self, chunk: &C) -> Result<(), LogsError> {
        let files = match (self.produce)(chunk) {
            Some(files) => files,
            None => return Ok(()),
        };

        // The last file is always the root — capture its path on first write
        if self.primary_path.is_none() {
            if let Some((path, _)) = files.last() {
                self.primary_path = Some(path.clone());
            }
        }

        // Filter out files whose content matches the buffer
        let changed: Vec<(String, Vec<u8>)> = files.into_iter().filter(|(path, bytes)| {
            if self.buffer.get(path).map_or(false, |prev| prev == bytes) {
                return false;
            }
            self.buffer.insert(path.clone(), bytes.clone());
            true
        }).collect();

        futures::future::try_join_all(changed.into_iter().map(|(path, bytes)| {
            let full_path = self.logs_dir.join(path);
            async move {
                if let Some(parent) = full_path.parent() {
                    tokio::fs::create_dir_all(parent).await
                        .map_err(|e| LogsError::Write(parent.to_path_buf(), e))?;
                }
                tokio::fs::write(&full_path, bytes).await
                    .map_err(|e| LogsError::Write(full_path, e))
            }
        })).await?;

        Ok(())
    }
}
