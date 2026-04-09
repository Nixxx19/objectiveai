use std::path::PathBuf;

use super::LogsError;

/// Writes streaming chunks to the log file structure on disk.
///
/// `C` is the chunk type. The `produce` closure extracts `(path, bytes)`
/// pairs from each chunk.
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

    /// Write a chunk to disk.
    pub fn write(&self, chunk: &C) -> Result<(), LogsError> {
        let files = match (self.produce)(chunk) {
            Some(files) => files,
            None => return Ok(()),
        };
        for (path, bytes) in files {
            let full_path = self.logs_dir.join(path);
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| LogsError::Write(parent.to_path_buf(), e))?;
            }
            std::fs::write(&full_path, bytes)
                .map_err(|e| LogsError::Write(full_path, e))?;
        }
        Ok(())
    }
}
