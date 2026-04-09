use std::path::PathBuf;

use super::{ListItem, LogsError};

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

    fn list_endpoint(&self, endpoint: &str) -> Result<Vec<ListItem>, LogsError> {
        let dir = self.endpoint_dir(endpoint);
        if !dir.exists() {
            return Ok(Vec::new());
        }
        let mut items = Vec::new();
        for entry in std::fs::read_dir(&dir)
            .map_err(|e| LogsError::ReadDir(dir.clone(), e))?
        {
            let entry = entry.map_err(|e| LogsError::ReadDir(dir.clone(), e))?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            let stem = match path.file_stem().and_then(|s| s.to_str()) {
                Some(s) => s.to_string(),
                None => continue,
            };
            let metadata = std::fs::metadata(&path)
                .map_err(|e| LogsError::Read(path.clone(), e))?;
            let created = metadata.modified()
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
                .duration_since(std::time::SystemTime::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            items.push(ListItem { id: stem, created });
        }
        items.sort_by(|a, b| b.created.cmp(&a.created));
        Ok(items)
    }

    // -----------------------------------------------------------------------
    // List methods
    // -----------------------------------------------------------------------

    pub fn list_agent_completions(&self) -> Result<Vec<ListItem>, LogsError> {
        self.list_endpoint("agent/completions")
    }

    pub fn list_vector_completions(&self) -> Result<Vec<ListItem>, LogsError> {
        self.list_endpoint("vector/completions")
    }

    pub fn list_function_executions(&self) -> Result<Vec<ListItem>, LogsError> {
        self.list_endpoint("functions/executions")
    }

    pub fn list_function_inventions(&self) -> Result<Vec<ListItem>, LogsError> {
        self.list_endpoint("functions/inventions")
    }

    pub fn list_function_inventions_recursive(&self) -> Result<Vec<ListItem>, LogsError> {
        self.list_endpoint("functions/inventions/recursive")
    }

    // pub fn list_function_profile_computations(&self) -> Result<Vec<ListItem>, LogsError> {
    //     self.list_endpoint("functions/profiles/computations")
    // }

    pub fn list_laboratory_executions(&self) -> Result<Vec<ListItem>, LogsError> {
        self.list_endpoint("laboratories/executions")
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
}
