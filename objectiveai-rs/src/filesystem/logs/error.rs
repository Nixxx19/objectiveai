#[derive(Debug, thiserror::Error)]
pub enum LogsError {
    #[error("failed to read log directory {0}: {1}")]
    ReadDir(std::path::PathBuf, std::io::Error),
    #[error("failed to read log file {0}: {1}")]
    Read(std::path::PathBuf, std::io::Error),
    #[error("failed to parse log file {0}: {1}")]
    Parse(std::path::PathBuf, serde_json::Error),
    #[error("failed to serialize log: {0}")]
    Serialize(serde_json::Error),
    #[error("failed to write log file {0}: {1}")]
    Write(std::path::PathBuf, std::io::Error),
    #[error("log not found: {0}")]
    NotFound(String),
}
