//! Generic unary + streaming helpers for the per-endpoint subcommands.
//!
//! Every leaf under `api/<...>/{post,get,delete}.rs` is one of:
//!
//! - `call_unary::<Req, Resp>(...)` — emit one notification, then return.
//! - `call_streaming::<Req, Chunk>(...)` — emit one notification per chunk.
//!
//! For no-body endpoints, `Req = ()` and `body = None`.

use futures::StreamExt;
use objectiveai_sdk::cli::output::{Handle, Notification, Output};

pub async fn call_unary<Req, Resp>(
    cli_config: &crate::Config,
    handle: &Handle,
    method: reqwest::Method,
    path: &str,
    body: Option<Req>,
) -> Result<(), crate::error::Error>
where
    Req: serde::Serialize + Send,
    Resp: serde::de::DeserializeOwned + serde::Serialize + Send + 'static,
{
    let (_client, mut config) = crate::config::read(cli_config).await?;
    let http = super::client::build_http_client(&mut config);
    let response: Resp = http.send_unary(method, path, body).await?;
    Output::<Resp>::Notification(Notification { value: response })
        .emit(handle)
        .await;
    Ok(())
}

pub async fn call_streaming<Req, Chunk>(
    cli_config: &crate::Config,
    handle: &Handle,
    method: reqwest::Method,
    path: &str,
    body: Option<Req>,
) -> Result<(), crate::error::Error>
where
    Req: serde::Serialize + Send,
    Chunk: serde::de::DeserializeOwned + serde::Serialize + Send + 'static,
{
    let (_client, mut config) = crate::config::read(cli_config).await?;
    let http = super::client::build_http_client(&mut config);
    let stream = http
        .send_streaming::<Chunk, _, _>(method, path.to_string(), body)
        .await?;
    let mut stream = std::pin::pin!(stream);
    while let Some(result) = stream.next().await {
        let chunk = result?;
        Output::<Chunk>::Notification(Notification { value: chunk })
            .emit(handle)
            .await;
    }
    Ok(())
}
