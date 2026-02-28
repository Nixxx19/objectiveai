/// Errors that can occur when communicating with upstream providers.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// No upstream provider is available.
    #[error("no upstream available")]
    NoUpstreamAvailable,
}

impl objectiveai::error::StatusError for Error {
    fn status(&self) -> u16 {
        match self {
            Error::NoUpstreamAvailable => 500,
        }
    }

    fn message(&self) -> Option<serde_json::Value> {
        match self {
            Error::NoUpstreamAvailable => Some(serde_json::json!({
                "kind": "no_upstream_available",
            })),
        }
    }
}
