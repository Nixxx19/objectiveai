#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("index {0} out of bounds (len {1})")]
    IndexOutOfBounds(usize, usize),
    #[error("remote {0:?} is not valid for configuration")]
    InvalidRemote(crate::Remote),
    #[error("failed to read config file {0}: {1}")]
    Read(std::path::PathBuf, std::io::Error),
    #[error("failed to parse config file {0}: {1}")]
    Parse(std::path::PathBuf, serde_json::Error),
    #[error("failed to serialize config: {0}")]
    Serialize(serde_json::Error),
    #[error("failed to write config file {0}: {1}")]
    Write(std::path::PathBuf, std::io::Error),
    #[error("jq parse error: {0}")]
    JqParse(String),
    #[error("jq compile error: {0}")]
    JqCompile(String),
    #[error("jq runtime error: {0}")]
    JqRuntime(String),
}
