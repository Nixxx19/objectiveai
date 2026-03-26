#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Config(#[from] objectiveai::config::ConfigError),
    #[error("viewer setup failed: {0}")]
    ViewerSetup(std::io::Error),
    #[error("api setup failed: {0}")]
    ApiSetup(std::io::Error),
    #[error("viewer config has secret but no signature, or signature but no secret")]
    ViewerSecretSignatureConfigMismatch,
    #[error("VIEWER_SECRET env var set without VIEWER_SIGNATURE, or vice versa")]
    ViewerSecretSignatureEnvMismatch,
    #[error("{0}")]
    Http(#[from] objectiveai::HttpError),
    #[error("filesystem source is not supported for function-profile pairs")]
    PairsFilesystemNotSupported,
    #[error("favorite not found: {0}")]
    FavoriteNotFound(String),
    #[error("{0}")]
    MissingArgs(&'static str),
    #[error("no python interpreter found (install Python or enable the rustpython feature)")]
    PythonNotFound,
    #[error("failed to read python file {0}: {1}")]
    PythonFileRead(std::path::PathBuf, std::io::Error),
}
