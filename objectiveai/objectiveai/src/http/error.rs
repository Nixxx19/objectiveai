use crate::error;

#[derive(thiserror::Error, Debug)]
pub enum HttpError {
    #[error("deserialization error: {0}")]
    DeserializationError(#[from] serde_path_to_error::Error<serde_json::Error>),
    #[error("received bad status code: {code}, body: {body}")]
    BadStatus {
        code: reqwest::StatusCode,
        body: serde_json::Value,
    },
    #[error("error fetching stream: {0}")]
    StreamError(#[from] reqwest_eventsource::Error),
    #[error("request error: {0}")]
    RequestError(reqwest::Error),
    #[error("streaming request error: {0}")]
    StreamingRequestError(#[from] reqwest_eventsource::CannotCloneRequestError),
    #[error("http error: {0}")]
    HttpError(reqwest::Error),
    #[error(transparent)]
    ApiError(#[from] error::ResponseError),
}
