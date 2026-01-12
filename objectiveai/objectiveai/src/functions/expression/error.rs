#[derive(Debug, thiserror::Error)]
pub enum ExpressionError {
    #[error(transparent)]
    JmespathError(#[from] jmespath::JmespathError),
    #[error(transparent)]
    DeserializationError(#[from] serde_json::Error),
    #[error("expected one value, found many")]
    ExpectedOneValueFoundMany,
}
