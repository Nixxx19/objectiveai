#[derive(Debug, PartialEq, thiserror::Error)]
pub enum Error {
    #[error("rate limited")]
    RateLimit,
}
