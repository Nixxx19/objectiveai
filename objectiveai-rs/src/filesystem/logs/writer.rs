use std::path::PathBuf;

use super::LogsError;

/// Writes streaming chunks to the log file structure on disk.
///
/// `C` is the chunk type. The `produce` function pointer extracts
/// `(path, bytes)` pairs from each chunk.
pub struct LogWriter<C> {
    logs_dir: PathBuf,
    produce: fn(&C) -> Option<Vec<(String, Vec<u8>)>>,
}

impl<C> LogWriter<C> {
    pub fn new(
        logs_dir: PathBuf,
        produce: fn(&C) -> Option<Vec<(String, Vec<u8>)>>,
    ) -> Self {
        Self { logs_dir, produce }
    }

    /// Write a chunk to disk. All files are written concurrently.
    pub async fn write(&self, chunk: &C) -> Result<(), LogsError> {
        let files = match (self.produce)(chunk) {
            Some(files) => files,
            None => return Ok(()),
        };

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
