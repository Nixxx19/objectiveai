use crate::{HttpClient, HttpError};
use futures::Stream;

pub async fn create_vector_completion_unary(
    client: &HttpClient,
    mut params: super::request::VectorCompletionCreateParams,
) -> Result<super::response::unary::VectorCompletion, HttpError> {
    params.stream = None;
    client
        .send_unary(reqwest::Method::POST, "vector/completions", Some(params))
        .await
}

pub async fn create_vector_completion_streaming(
    client: &HttpClient,
    mut params: super::request::VectorCompletionCreateParams,
) -> Result<
    impl Stream<
        Item = Result<
            super::response::streaming::VectorCompletionChunk,
            HttpError,
        >,
    >,
    HttpError,
> {
    params.stream = Some(true);
    client
        .send_streaming(
            reqwest::Method::POST,
            "vector/completions",
            Some(params),
        )
        .await
}
