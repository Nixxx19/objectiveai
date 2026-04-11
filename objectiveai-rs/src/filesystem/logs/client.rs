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
        self.list_endpoint("agent/completions", offset, limit).await
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
        self.clear_endpoint("agent/completions").await
    }

    pub async fn clear_vector_completions(&self) -> Result<u64, LogsError> {
        self.clear_endpoint("vector/completions").await
    }

    pub async fn clear_function_executions(&self) -> Result<u64, LogsError> {
        self.clear_endpoint("functions/executions").await
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

    async fn read_json(&self, path: &str, filename: &str) -> Result<serde_json::Value, LogsError> {
        let full = self.logs_dir().join(path).join(format!("{filename}.json"));
        let bytes = tokio::fs::read(&full).await
            .map_err(|e| LogsError::Read(full.clone(), e))?;
        serde_json::from_slice(&bytes)
            .map_err(|e| LogsError::Parse(full, e))
    }

    async fn read_data_url(&self, path: &str, filename: &str) -> Result<String, LogsError> {
        use base64::Engine;
        let full = self.logs_dir().join(path).join(filename);
        let bytes = tokio::fs::read(&full).await
            .map_err(|e| LogsError::Read(full.clone(), e))?;
        let mime = mime_guess::from_path(&full)
            .first_or_octet_stream()
            .to_string();
        let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
        Ok(format!("data:{mime};base64,{b64}"))
    }

    // -----------------------------------------------------------------------
    // Read methods — agent completions
    // -----------------------------------------------------------------------

    pub async fn read_agent_completion(&self, filename: &str) -> Result<serde_json::Value, LogsError> {
        self.read_json("agent/completions", filename).await
    }

    pub async fn read_agent_completion_continuation(&self, filename: &str) -> Result<serde_json::Value, LogsError> {
        self.read_json("agent/completions/continuation", filename).await
    }

    pub async fn read_agent_completion_message(&self, filename: &str) -> Result<serde_json::Value, LogsError> {
        self.read_json("agent/completions/messages", filename).await
    }

    pub async fn read_agent_completion_message_logprobs(&self, filename: &str) -> Result<serde_json::Value, LogsError> {
        self.read_json("agent/completions/messages/logprobs", filename).await
    }

    pub async fn read_agent_completion_message_image(&self, filename: &str) -> Result<String, LogsError> {
        self.read_data_url("agent/completions/messages/image", filename).await
    }

    pub async fn read_agent_completion_message_audio(&self, filename: &str) -> Result<String, LogsError> {
        self.read_data_url("agent/completions/messages/audio", filename).await
    }

    pub async fn read_agent_completion_message_video(&self, filename: &str) -> Result<String, LogsError> {
        self.read_data_url("agent/completions/messages/video", filename).await
    }

    pub async fn read_agent_completion_message_file(&self, filename: &str) -> Result<String, LogsError> {
        self.read_data_url("agent/completions/messages/file", filename).await
    }

    // -----------------------------------------------------------------------
    // Read methods — vector completions
    // -----------------------------------------------------------------------

    pub async fn read_vector_completion(&self, filename: &str) -> Result<serde_json::Value, LogsError> {
        self.read_json("vector/completions", filename).await
    }

    // -----------------------------------------------------------------------
    // Read methods — function executions
    // -----------------------------------------------------------------------

    pub async fn read_function_execution(&self, filename: &str) -> Result<serde_json::Value, LogsError> {
        self.read_json("functions/executions", filename).await
    }

    pub async fn read_function_execution_retry_token(&self, filename: &str) -> Result<serde_json::Value, LogsError> {
        self.read_json("functions/executions/retry_token", filename).await
    }

    // -----------------------------------------------------------------------
    // Read methods — function inventions
    // -----------------------------------------------------------------------

    pub async fn read_function_invention(&self, filename: &str) -> Result<serde_json::Value, LogsError> {
        self.read_json("functions/inventions", filename).await
    }

    // -----------------------------------------------------------------------
    // Read methods — function inventions recursive
    // -----------------------------------------------------------------------

    pub async fn read_function_invention_recursive(&self, filename: &str) -> Result<serde_json::Value, LogsError> {
        self.read_json("functions/inventions/recursive", filename).await
    }

    // -----------------------------------------------------------------------
    // Read methods — laboratory executions
    // -----------------------------------------------------------------------

    pub async fn read_laboratory_execution(&self, filename: &str) -> Result<serde_json::Value, LogsError> {
        self.read_json("laboratories/executions", filename).await
    }

    // -----------------------------------------------------------------------
    // Generic read
    // -----------------------------------------------------------------------

    /// Read any log file by its full path (relative to `logs/`).
    ///
    /// The path is split into the directory portion and filename.
    /// Returns `LogContent::Json` for JSON files, `LogContent::Bytes` for media.
    pub async fn read(&self, path: &str) -> Result<LogContent, LogsError> {
        let (dir, filename) = path.rsplit_once('/')
            .ok_or_else(|| LogsError::InvalidPath(path.to_string()))?;

        match dir {
            "agent/completions" => self.read_agent_completion(filename).await.map(LogContent::Json),
            "agent/completions/continuation" => self.read_agent_completion_continuation(filename).await.map(LogContent::Json),
            "agent/completions/messages" => self.read_agent_completion_message(filename).await.map(LogContent::Json),
            "agent/completions/messages/logprobs" => self.read_agent_completion_message_logprobs(filename).await.map(LogContent::Json),
            "agent/completions/messages/image" => self.read_agent_completion_message_image(filename).await.map(LogContent::DataUrl),
            "agent/completions/messages/audio" => self.read_agent_completion_message_audio(filename).await.map(LogContent::DataUrl),
            "agent/completions/messages/video" => self.read_agent_completion_message_video(filename).await.map(LogContent::DataUrl),
            "agent/completions/messages/file" => self.read_agent_completion_message_file(filename).await.map(LogContent::DataUrl),
            "vector/completions" => self.read_vector_completion(filename).await.map(LogContent::Json),
            "functions/executions" => self.read_function_execution(filename).await.map(LogContent::Json),
            "functions/executions/retry_token" => self.read_function_execution_retry_token(filename).await.map(LogContent::Json),
            "functions/inventions" => self.read_function_invention(filename).await.map(LogContent::Json),
            "functions/inventions/recursive" => self.read_function_invention_recursive(filename).await.map(LogContent::Json),
            "laboratories/executions" => self.read_laboratory_execution(filename).await.map(LogContent::Json),
            _ => Err(LogsError::InvalidPath(path.to_string())),
        }
    }
}
