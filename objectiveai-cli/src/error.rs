#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Config(#[from] objectiveai::ConfigError),
    #[error("viewer setup failed: {0}")]
    ViewerSetup(std::io::Error),
    #[error("api setup failed: {0}")]
    ApiSetup(std::io::Error),
}
