use crate::{HttpClient, HttpError};
use futures::Stream;

pub async fn create_chat_completion_unary(
    client: &HttpClient,
    mut params: super::request::ChatCompletionCreateParams,
) -> Result<super::response::unary::ChatCompletion, HttpError> {
    params.stream = None;
    client
        .send_unary(reqwest::Method::POST, "chat/completions", Some(params))
        .await
}

pub async fn create_chat_completion_streaming(
    client: &HttpClient,
    mut params: super::request::ChatCompletionCreateParams,
) -> Result<
    impl Stream<
        Item = Result<
            super::response::streaming::ChatCompletionChunk,
            HttpError,
        >,
    >,
    HttpError,
> {
    params.stream = Some(true);
    client
        .send_streaming(reqwest::Method::POST, "chat/completions", Some(params))
        .await
}
