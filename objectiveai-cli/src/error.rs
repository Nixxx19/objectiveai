#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Config(#[from] objectiveai::ConfigError),
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
}
