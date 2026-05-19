//! `objectiveai viewer send <path> <body>` — POST a JSON body to the
//! viewer's HTTP server and await the response.
//!
//! Reads `api.headers.x_viewer_address` and
//! `api.headers.x_viewer_signature` from the filesystem config (the
//! canonical "where to send + what to sign with" pair, settable via
//! `objectiveai api headers x-viewer-{address,signature} set`).
//! Bypasses the SDK's `http::viewer::Client` because that client is
//! fire-and-forget — this command needs a synchronous send so the
//! caller can see the viewer's response.

use objectiveai_sdk::cli::output::{Handle, Notification, Output};

#[derive(serde::Serialize)]
struct ViewerSendResult {
    status: u16,
    body: serde_json::Value,
}

pub async fn run(
    cli_config: &crate::Config,
    handle: &Handle,
    path: &str,
    body: &str,
) -> Result<(), crate::error::Error> {
    let fs_client = objectiveai_sdk::filesystem::Client::new(
        cli_config.config_base_dir.as_deref(),
        cli_config.commit_author_name.as_deref(),
        cli_config.commit_author_email.as_deref(),
    );
    let mut config = fs_client.read_config().await?;

    let address = config
        .api()
        .headers()
        .get_x_viewer_address()
        .ok_or(crate::error::Error::ViewerAddressNotConfigured)?
        .to_string();
    let signature = config
        .api()
        .headers()
        .get_x_viewer_signature()
        .map(str::to_string);

    if !path.starts_with('/') {
        return Err(crate::error::Error::ViewerPathMissingSlash(path.to_string()));
    }
    let url = format!("{}{}", address.trim_end_matches('/'), path);

    // Validate JSON well-formedness before send. We re-emit the parsed
    // value (not the raw `body` string) so trailing whitespace / odd
    // formatting in the user's input doesn't reach the viewer.
    let parsed: serde_json::Value = serde_json::from_str(body)
        .map_err(|e| crate::error::Error::ViewerBodyJsonParse(e.to_string()))?;

    let http_client = reqwest::Client::new();
    let mut req = http_client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&parsed);
    if let Some(sig) = signature.as_deref() {
        req = req.header("X-VIEWER-SIGNATURE", sig);
    }
    let response = req
        .send()
        .await
        .map_err(|e| crate::error::Error::ViewerSendHttp(e.to_string()))?;

    let status = response.status();
    let response_text = response.text().await.unwrap_or_default();
    let response_body = serde_json::from_str::<serde_json::Value>(&response_text)
        .unwrap_or_else(|_| serde_json::Value::String(response_text));

    if status.is_success() {
        Output::<ViewerSendResult>::Notification(Notification {
            value: ViewerSendResult {
                status: status.as_u16(),
                body: response_body,
            },
        })
        .emit(handle)
        .await;
        Ok(())
    } else {
        Err(crate::error::Error::ViewerSendBadStatus {
            status: status.as_u16(),
            body: response_body.to_string(),
        })
    }
}
