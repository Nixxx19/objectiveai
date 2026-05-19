//! `objectiveai viewer send <path> <body>` — POST a JSON body to the
//! viewer's HTTP server and await the response.
//!
//! Branches on `config.viewer().get_mode()`:
//! - **Remote**: reads `api.headers.x_viewer_address` and
//!   `api.headers.x_viewer_signature` from the filesystem config and POSTs
//!   there. Same source the api server's viewer client uses.
//! - **Local**: spawns the embedded viewer subprocess (via the same
//!   helpers `api/run.rs` uses), POSTs to its bound address with the
//!   locally-configured signature, then kills the subprocess — matching
//!   the spawn-kill lifecycle at `api/run.rs:97-99`.
//!
//! Bypasses the SDK's `http::viewer::Client` because that client is
//! fire-and-forget — this command needs a synchronous send so the caller
//! can see the viewer's response.

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

    if !path.starts_with('/') {
        return Err(crate::error::Error::ViewerPathMissingSlash(path.to_string()));
    }
    let parsed: serde_json::Value = serde_json::from_str(body)
        .map_err(|e| crate::error::Error::ViewerBodyJsonParse(e.to_string()))?;

    let mode = config.viewer().get_mode();
    match mode {
        objectiveai_sdk::filesystem::config::ViewerMode::Remote => {
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
            do_post(&address, path, signature.as_deref(), &parsed, handle).await
        }
        #[cfg(feature = "viewer")]
        objectiveai_sdk::filesystem::config::ViewerMode::Local => {
            let (secret, _from_env, signature) =
                crate::api::resolve_viewer_secret(&mut config)?;
            let (mut child, viewer_addr_str) =
                crate::api::spawn_viewer(secret.as_deref(), &mut config).await?;
            let result =
                do_post(&viewer_addr_str, path, signature.as_deref(), &parsed, handle).await;
            let _ = child.kill().await;
            result
        }
        #[cfg(not(feature = "viewer"))]
        objectiveai_sdk::filesystem::config::ViewerMode::Local => {
            Err(crate::error::Error::LocalViewerFeatureDisabled)
        }
    }
}

async fn do_post(
    address: &str,
    path: &str,
    signature: Option<&str>,
    body: &serde_json::Value,
    handle: &Handle,
) -> Result<(), crate::error::Error> {
    let url = format!("{}{}", address.trim_end_matches('/'), path);

    let http_client = reqwest::Client::new();
    let mut req = http_client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(body);
    if let Some(sig) = signature {
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
