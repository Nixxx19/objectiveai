use std::path::PathBuf;

use super::LogsError;

/// Writes streaming chunks to the log file structure on disk.
///
/// `C` is the chunk type. The `produce` function pointer extracts
/// `(path, bytes)` pairs from each chunk.
pub struct LogWriter<C> {
    logs_dir: PathBuf,
    produce: fn(&C) -> Option<Vec<(String, Vec<u8>)>>,
    primary_path: std::sync::OnceLock<String>,
}

impl<C> LogWriter<C> {
    pub fn new(
        logs_dir: PathBuf,
        produce: fn(&C) -> Option<Vec<(String, Vec<u8>)>>,
    ) -> Self {
        Self { logs_dir, produce, primary_path: std::sync::OnceLock::new() }
    }

    /// The path of the primary (root) log file, relative to the logs directory.
    ///
    /// Returns `None` until at least one chunk has been written.
    pub fn primary_path(&self) -> Option<&str> {
        self.primary_path.get().map(|s| s.as_str())
    }

    /// Write a chunk to disk. All files are written concurrently.
    pub async fn write(&self, chunk: &C) -> Result<(), LogsError> {
        let files = match (self.produce)(chunk) {
            Some(files) => files,
            None => return Ok(()),
        };

        // The last file is always the root — capture its path on first write
        if let Some((path, _)) = files.last() {
            let _ = self.primary_path.set(path.clone());
        }

        futures::future::try_join_all(files.into_iter().map(|(path, bytes)| {
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
