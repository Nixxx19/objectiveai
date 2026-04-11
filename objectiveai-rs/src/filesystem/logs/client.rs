use std::path::PathBuf;

use super::{ListItem, LogsError};

/// Result of reading a log file — either parsed JSON or a data URL.
#[derive(Debug)]
pub enum LogContent {
    Json(serde_json::Value),
    /// A `data:{mime};base64,{payload}` string.
    DataUrl(String),
}

#[derive(Debug, Clone)]
pub struct LogsClient {
    base_dir: PathBuf,
}

impl LogsClient {
    pub fn new(base_dir: Option<impl Into<PathBuf>>) -> Self {
        let base_dir = match base_dir {
            Some(dir) => dir.into(),
            None => {
                #[cfg(feature = "env")]
                if let Ok(dir) = std::env::var("LOGS_BASE_DIR") {
                    return Self { base_dir: PathBuf::from(dir) };
                }
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".objectiveai")
            }
        };
        Self { base_dir }
    }

    fn logs_dir(&self) -> PathBuf {
        self.base_dir.join("logs")
    }

    fn endpoint_dir(&self, endpoint: &str) -> PathBuf {
        let mut dir = self.logs_dir();
        for segment in endpoint.split('/') {
            dir = dir.join(segment);
        }
        dir
    }

    async fn list_endpoint(&self, endpoint: &str, offset: usize, limit: usize) -> Result<Vec<ListItem>, LogsError> {
        let dir = self.endpoint_dir(endpoint);
        match tokio::fs::metadata(&dir).await {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(LogsError::ReadDir(dir, e)),
            Ok(_) => {}
        }
        let mut read_dir = tokio::fs::read_dir(&dir).await
            .map_err(|e| LogsError::ReadDir(dir.clone(), e))?;
        let mut items = Vec::new();
        while let Some(entry) = read_dir.next_entry().await
            .map_err(|e| LogsError::ReadDir(dir.clone(), e))?
        {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let stem = match path.file_stem().and_then(|s| s.to_str()) {
                Some(s) => s.to_string(),
                None => continue,
            };
            let metadata = tokio::fs::metadata(&path).await
                .map_err(|e| LogsError::Read(path.clone(), e))?;
            let created = metadata.modified()
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            items.push(ListItem { id: stem, created });
        }
        items.sort_by(|a, b| b.created.cmp(&a.created));
        if offset > 0 || limit < usize::MAX {
            items = items.into_iter().skip(offset).take(limit).collect();
        }
        Ok(items)
    }

    // -----------------------------------------------------------------------
    // List methods
    // -----------------------------------------------------------------------

    pub async fn list_agent_completions(&self, offset: usize, limit: usize) -> Result<Vec<ListItem>, LogsError> {
        self.list_endpoint("agents/completions", offset, limit).await
    }

    pub async fn list_vector_completions(&self, offset: usize, limit: usize) -> Result<Vec<ListItem>, LogsError> {
        self.list_endpoint("vector/completions", offset, limit).await
    }

    pub async fn list_function_executions(&self, offset: usize, limit: usize) -> Result<Vec<ListItem>, LogsError> {
        self.list_endpoint("functions/executions", offset, limit).await
    }

    pub async fn list_function_inventions(&self, offset: usize, limit: usize) -> Result<Vec<ListItem>, LogsError> {
        self.list_endpoint("functions/inventions", offset, limit).await
    }

    pub async fn list_function_inventions_recursive(&self, offset: usize, limit: usize) -> Result<Vec<ListItem>, LogsError> {
        self.list_endpoint("functions/inventions/recursive", offset, limit).await
    }

    // pub async fn list_function_profile_computations(&self, offset: usize, limit: usize) -> Result<Vec<ListItem>, LogsError> {
    //     self.list_endpoint("functions/profiles/computations", offset, limit).await
    // }

    pub async fn list_laboratory_executions(&self, offset: usize, limit: usize) -> Result<Vec<ListItem>, LogsError> {
        self.list_endpoint("laboratories/executions", offset, limit).await
    }

    // -----------------------------------------------------------------------
    // Clear methods
    // -----------------------------------------------------------------------

    /// Deletes all files (not subdirectories) in the given endpoint directory.
    async fn clear_endpoint(&self, endpoint: &str) -> Result<u64, LogsError> {
        let dir = self.endpoint_dir(endpoint);
        match tokio::fs::metadata(&dir).await {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(0),
            Err(e) => return Err(LogsError::ReadDir(dir, e)),
            Ok(_) => {}
        }
        let mut read_dir = tokio::fs::read_dir(&dir).await
            .map_err(|e| LogsError::ReadDir(dir.clone(), e))?;
        let mut count = 0u64;
        while let Some(entry) = read_dir.next_entry().await
            .map_err(|e| LogsError::ReadDir(dir.clone(), e))?
        {
            let path = entry.path();
            if path.is_file() {
                tokio::fs::remove_file(&path).await
                    .map_err(|e| LogsError::Read(path, e))?;
                count += 1;
            }
        }
        Ok(count)
    }

    pub async fn clear_agent_completions(&self) -> Result<u64, LogsError> {
        self.clear_endpoint("agents/completions").await
    }

    pub async fn clear_agent_completion_continuations(&self) -> Result<u64, LogsError> {
        self.clear_endpoint("agent/completions/continuation").await
    }

    pub async fn clear_agent_completion_messages(&self) -> Result<u64, LogsError> {
        self.clear_endpoint("agent/completions/messages").await
    }

    pub async fn clear_agent_completion_message_logprobs(&self) -> Result<u64, LogsError> {
        self.clear_endpoint("agent/completions/messages/logprobs").await
    }

    pub async fn clear_agent_completion_message_images(&self) -> Result<u64, LogsError> {
        self.clear_endpoint("agent/completions/messages/image").await
    }

    pub async fn clear_agent_completion_message_audio(&self) -> Result<u64, LogsError> {
        self.clear_endpoint("agent/completions/messages/audio").await
    }

    pub async fn clear_agent_completion_message_video(&self) -> Result<u64, LogsError> {
        self.clear_endpoint("agent/completions/messages/video").await
    }

    pub async fn clear_agent_completion_message_files(&self) -> Result<u64, LogsError> {
        self.clear_endpoint("agent/completions/messages/file").await
    }

    pub async fn clear_vector_completions(&self) -> Result<u64, LogsError> {
        self.clear_endpoint("vector/completions").await
    }

    pub async fn clear_function_executions(&self) -> Result<u64, LogsError> {
        self.clear_endpoint("functions/executions").await
    }

    pub async fn clear_function_execution_retry_tokens(&self) -> Result<u64, LogsError> {
        self.clear_endpoint("functions/executions/retry_token").await
    }

    pub async fn clear_function_inventions(&self) -> Result<u64, LogsError> {
        self.clear_endpoint("functions/inventions").await
    }

    pub async fn clear_function_inventions_recursive(&self) -> Result<u64, LogsError> {
        self.clear_endpoint("functions/inventions/recursive").await
    }

    pub async fn clear_laboratory_executions(&self) -> Result<u64, LogsError> {
        self.clear_endpoint("laboratories/executions").await
    }

    // -----------------------------------------------------------------------
    // Write methods
    // -----------------------------------------------------------------------

    pub fn write_agent_completion(&self) -> super::LogWriter<crate::agent::completions::response::streaming::AgentCompletionChunk> {
        super::LogWriter::new(self.logs_dir(), |chunk| chunk.produce_files().map(|(_, files)| files))
    }

    pub fn write_vector_completion(&self) -> super::LogWriter<crate::vector::completions::response::streaming::VectorCompletionChunk> {
        super::LogWriter::new(self.logs_dir(), |chunk| chunk.produce_files().map(|(_, files)| files))
    }

    pub fn write_function_execution(&self) -> super::LogWriter<crate::functions::executions::response::streaming::FunctionExecutionChunk> {
        super::LogWriter::new(self.logs_dir(), |chunk| chunk.produce_files().map(|(_, files)| files))
    }

    pub fn write_function_invention(&self) -> super::LogWriter<crate::functions::inventions::response::streaming::FunctionInventionChunk> {
        super::LogWriter::new(self.logs_dir(), |chunk| chunk.produce_files().map(|(_, files)| files))
    }

    pub fn write_function_invention_recursive(&self) -> super::LogWriter<crate::functions::inventions::recursive::response::streaming::FunctionInventionRecursiveChunk> {
        super::LogWriter::new(self.logs_dir(), |chunk| chunk.produce_files().map(|(_, files)| files))
    }

    // pub fn write_function_profile_computation(&self) -> super::LogWriter<crate::functions::profiles::computations::response::streaming::FunctionProfileComputationChunk> {
    //     super::LogWriter::new(self.logs_dir(), |chunk| chunk.produce_files().map(|(_, files)| files))
    // }

    pub fn write_laboratory_execution(&self) -> super::LogWriter<crate::laboratories::executions::response::streaming::LaboratoryExecutionChunk> {
        super::LogWriter::new(self.logs_dir(), |chunk| chunk.produce_files().map(|(_, files)| files))
    }

    // -----------------------------------------------------------------------
    // Read helpers
    // -----------------------------------------------------------------------

    async fn read_json(&self, dir: &str, stem: &str) -> Result<serde_json::Value, LogsError> {
        let full = self.logs_dir().join(dir).join(format!("{stem}.json"));
        let bytes = tokio::fs::read(&full).await
            .map_err(|e| LogsError::Read(full.clone(), e))?;
        serde_json::from_slice(&bytes)
            .map_err(|e| LogsError::Parse(full, e))
    }

    /// Finds the first file in `dir` whose name starts with `stem.` (any extension)
    /// and returns it as a data URL.
    async fn read_data_url_by_stem(&self, dir: &str, stem: &str) -> Result<String, LogsError> {
        use base64::Engine;
        let dir_path = self.logs_dir().join(dir);
        let prefix = format!("{stem}.");
        let mut read_dir = tokio::fs::read_dir(&dir_path).await
            .map_err(|e| LogsError::ReadDir(dir_path.clone(), e))?;
        while let Some(entry) = read_dir.next_entry().await
            .map_err(|e| LogsError::ReadDir(dir_path.clone(), e))?
        {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with(&prefix) {
                    let bytes = tokio::fs::read(&path).await
                        .map_err(|e| LogsError::Read(path.clone(), e))?;
                    let mime = mime_guess::from_path(&path)
                        .first_or_octet_stream()
                        .to_string();
                    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                    return Ok(format!("data:{mime};base64,{b64}"));
                }
            }
        }
        Err(LogsError::Read(dir_path.join(&prefix), std::io::Error::new(std::io::ErrorKind::NotFound, "no matching file")))
    }

    // -----------------------------------------------------------------------
    // Read methods — agent completions
    // -----------------------------------------------------------------------

    pub async fn read_agent_completion(&self, id: &str) -> Result<serde_json::Value, LogsError> {
        self.read_json("agents/completions", id).await
    }

    pub async fn read_agent_completion_continuation(&self, id: &str) -> Result<serde_json::Value, LogsError> {
        self.read_json("agents/completions/continuation", id).await
    }

    pub async fn read_agent_completion_message(&self, id: &str, message_index: u64) -> Result<serde_json::Value, LogsError> {
        self.read_json("agents/completions/messages", &format!("{id}_{message_index}")).await
    }

    pub async fn read_agent_completion_message_logprobs(&self, id: &str, message_index: u64) -> Result<serde_json::Value, LogsError> {
        self.read_json("agents/completions/messages/logprobs", &format!("{id}_{message_index}")).await
    }

    pub async fn read_agent_completion_message_image(&self, id: &str, message_index: u64, media_index: u64) -> Result<String, LogsError> {
        self.read_data_url_by_stem("agents/completions/messages/image", &format!("{id}_{message_index}_{media_index}")).await
    }

    pub async fn read_agent_completion_message_audio(&self, id: &str, message_index: u64, media_index: u64) -> Result<String, LogsError> {
        self.read_data_url_by_stem("agents/completions/messages/audio", &format!("{id}_{message_index}_{media_index}")).await
    }

    pub async fn read_agent_completion_message_video(&self, id: &str, message_index: u64, media_index: u64) -> Result<String, LogsError> {
        self.read_data_url_by_stem("agents/completions/messages/video", &format!("{id}_{message_index}_{media_index}")).await
    }

    pub async fn read_agent_completion_message_file(&self, id: &str, message_index: u64, media_index: u64) -> Result<String, LogsError> {
        self.read_data_url_by_stem("agents/completions/messages/file", &format!("{id}_{message_index}_{media_index}")).await
    }

    // -----------------------------------------------------------------------
    // Read methods — vector completions
    // -----------------------------------------------------------------------

    pub async fn read_vector_completion(&self, id: &str) -> Result<serde_json::Value, LogsError> {
        self.read_json("vector/completions", id).await
    }

    // -----------------------------------------------------------------------
    // Read methods — function executions
    // -----------------------------------------------------------------------

    pub async fn read_function_execution(&self, id: &str) -> Result<serde_json::Value, LogsError> {
        self.read_json("functions/executions", id).await
    }

    pub async fn read_function_execution_retry_token(&self, id: &str) -> Result<serde_json::Value, LogsError> {
        self.read_json("functions/executions/retry_token", id).await
    }

    // -----------------------------------------------------------------------
    // Read methods — function inventions
    // -----------------------------------------------------------------------

    pub async fn read_function_invention(&self, id: &str) -> Result<serde_json::Value, LogsError> {
        self.read_json("functions/inventions", id).await
    }

    // -----------------------------------------------------------------------
    // Read methods — function inventions recursive
    // -----------------------------------------------------------------------

    pub async fn read_function_invention_recursive(&self, id: &str) -> Result<serde_json::Value, LogsError> {
        self.read_json("functions/inventions/recursive", id).await
    }

    // -----------------------------------------------------------------------
    // Read methods — laboratory executions
    // -----------------------------------------------------------------------

    pub async fn read_laboratory_execution(&self, id: &str) -> Result<serde_json::Value, LogsError> {
        self.read_json("laboratories/executions", id).await
    }

    // -----------------------------------------------------------------------
    // Subscribe helpers
    // -----------------------------------------------------------------------

    /// Polls for a JSON file to be created or modified. Returns `Some(value)`
    /// on change, `None` on deletion or timeout.
    async fn subscribe_json(
        &self,
        dir: &str,
        stem: &str,
        timeout: std::time::Duration,
    ) -> Option<serde_json::Value> {
        let full = self.logs_dir().join(dir).join(format!("{stem}.json"));
        self.poll_file(&full, timeout).await?;
        let bytes = tokio::fs::read(&full).await.ok()?;
        serde_json::from_slice(&bytes).ok()
    }

    /// Polls for a media file (any extension matching `stem.`) to be created
    /// or modified. Returns `Some(data_url)` on change, `None` on deletion or timeout.
    async fn subscribe_data_url_by_stem(
        &self,
        dir: &str,
        stem: &str,
        timeout: std::time::Duration,
    ) -> Option<String> {
        use base64::Engine;
        let dir_path = self.logs_dir().join(dir);
        let prefix = format!("{stem}.");

        let deadline = tokio::time::Instant::now() + timeout;
        let initial_mtime = self.find_file_mtime_by_prefix(&dir_path, &prefix).await;

        loop {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            if tokio::time::Instant::now() >= deadline {
                return None;
            }

            let current_mtime = self.find_file_mtime_by_prefix(&dir_path, &prefix).await;
            match (&initial_mtime, &current_mtime) {
                // File appeared
                (None, Some((path, _))) => {
                    let bytes = tokio::fs::read(path).await.ok()?;
                    let mime = mime_guess::from_path(path).first_or_octet_stream().to_string();
                    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                    return Some(format!("data:{mime};base64,{b64}"));
                }
                // File modified
                (Some((_, old_t)), Some((path, new_t))) if new_t > old_t => {
                    let bytes = tokio::fs::read(path).await.ok()?;
                    let mime = mime_guess::from_path(path).first_or_octet_stream().to_string();
                    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                    return Some(format!("data:{mime};base64,{b64}"));
                }
                // File deleted
                (Some(_), None) => return None,
                // No change yet
                _ => continue,
            }
        }
    }

    /// Polls a specific file path for creation, modification, or deletion.
    /// Returns `Some(())` on create/modify, `None` on deletion or timeout.
    async fn poll_file(
        &self,
        path: &std::path::Path,
        timeout: std::time::Duration,
    ) -> Option<()> {
        let deadline = tokio::time::Instant::now() + timeout;
        let initial_mtime = Self::file_mtime(path).await;

        loop {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            if tokio::time::Instant::now() >= deadline {
                return None;
            }

            let current_mtime = Self::file_mtime(path).await;
            match (&initial_mtime, &current_mtime) {
                (None, Some(_)) => return Some(()),     // created
                (Some(old), Some(new)) if new > old => return Some(()), // modified
                (Some(_), None) => return None,         // deleted
                _ => continue,                          // no change
            }
        }
    }

    async fn file_mtime(path: &std::path::Path) -> Option<std::time::SystemTime> {
        tokio::fs::metadata(path).await.ok()?.modified().ok()
    }

    async fn find_file_mtime_by_prefix(
        &self,
        dir: &std::path::Path,
        prefix: &str,
    ) -> Option<(std::path::PathBuf, std::time::SystemTime)> {
        let mut read_dir = tokio::fs::read_dir(dir).await.ok()?;
        while let Some(entry) = read_dir.next_entry().await.ok()? {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if name.starts_with(prefix) {
                    let mtime = tokio::fs::metadata(&path).await.ok()?.modified().ok()?;
                    return Some((path, mtime));
                }
            }
        }
        None
    }

    // -----------------------------------------------------------------------
    // Subscribe methods — agent completions
    // -----------------------------------------------------------------------

    pub async fn subscribe_agent_completion(&self, id: &str, timeout: std::time::Duration) -> Option<serde_json::Value> {
        self.subscribe_json("agents/completions", id, timeout).await
    }

    pub async fn subscribe_agent_completion_continuation(&self, id: &str, timeout: std::time::Duration) -> Option<serde_json::Value> {
        self.subscribe_json("agents/completions/continuation", id, timeout).await
    }

    pub async fn subscribe_agent_completion_message(&self, id: &str, message_index: u64, timeout: std::time::Duration) -> Option<serde_json::Value> {
        self.subscribe_json("agents/completions/messages", &format!("{id}_{message_index}"), timeout).await
    }

    pub async fn subscribe_agent_completion_message_logprobs(&self, id: &str, message_index: u64, timeout: std::time::Duration) -> Option<serde_json::Value> {
        self.subscribe_json("agents/completions/messages/logprobs", &format!("{id}_{message_index}"), timeout).await
    }

    pub async fn subscribe_agent_completion_message_image(&self, id: &str, message_index: u64, media_index: u64, timeout: std::time::Duration) -> Option<String> {
        self.subscribe_data_url_by_stem("agents/completions/messages/image", &format!("{id}_{message_index}_{media_index}"), timeout).await
    }

    pub async fn subscribe_agent_completion_message_audio(&self, id: &str, message_index: u64, media_index: u64, timeout: std::time::Duration) -> Option<String> {
        self.subscribe_data_url_by_stem("agents/completions/messages/audio", &format!("{id}_{message_index}_{media_index}"), timeout).await
    }

    pub async fn subscribe_agent_completion_message_video(&self, id: &str, message_index: u64, media_index: u64, timeout: std::time::Duration) -> Option<String> {
        self.subscribe_data_url_by_stem("agents/completions/messages/video", &format!("{id}_{message_index}_{media_index}"), timeout).await
    }

    pub async fn subscribe_agent_completion_message_file(&self, id: &str, message_index: u64, media_index: u64, timeout: std::time::Duration) -> Option<String> {
        self.subscribe_data_url_by_stem("agents/completions/messages/file", &format!("{id}_{message_index}_{media_index}"), timeout).await
    }

    // -----------------------------------------------------------------------
    // Subscribe methods — vector completions
    // -----------------------------------------------------------------------

    pub async fn subscribe_vector_completion(&self, id: &str, timeout: std::time::Duration) -> Option<serde_json::Value> {
        self.subscribe_json("vector/completions", id, timeout).await
    }

    // -----------------------------------------------------------------------
    // Subscribe methods — function executions
    // -----------------------------------------------------------------------

    pub async fn subscribe_function_execution(&self, id: &str, timeout: std::time::Duration) -> Option<serde_json::Value> {
        self.subscribe_json("functions/executions", id, timeout).await
    }

    pub async fn subscribe_function_execution_retry_token(&self, id: &str, timeout: std::time::Duration) -> Option<serde_json::Value> {
        self.subscribe_json("functions/executions/retry_token", id, timeout).await
    }

    // -----------------------------------------------------------------------
    // Subscribe methods — function inventions
    // -----------------------------------------------------------------------

    pub async fn subscribe_function_invention(&self, id: &str, timeout: std::time::Duration) -> Option<serde_json::Value> {
        self.subscribe_json("functions/inventions", id, timeout).await
    }

    // -----------------------------------------------------------------------
    // Subscribe methods — function inventions recursive
    // -----------------------------------------------------------------------

    pub async fn subscribe_function_invention_recursive(&self, id: &str, timeout: std::time::Duration) -> Option<serde_json::Value> {
        self.subscribe_json("functions/inventions/recursive", id, timeout).await
    }

    // -----------------------------------------------------------------------
    // Subscribe methods — laboratory executions
    // -----------------------------------------------------------------------

    pub async fn subscribe_laboratory_execution(&self, id: &str, timeout: std::time::Duration) -> Option<serde_json::Value> {
        self.subscribe_json("laboratories/executions", id, timeout).await
    }
}
