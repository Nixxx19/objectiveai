use std::future::Future;
use std::sync::Arc;
use envconfig::Envconfig;

/// Reads config and dispatches to the appropriate run variant
/// based on API mode (Local/Remote) and Viewer mode (Local/Remote).
pub async fn run<F, Fut>(task: F) -> Result<crate::Output, crate::error::Error>
where
    F: FnOnce(objectiveai::HttpClient) -> Fut + Send + 'static,
    Fut: Future<Output = Result<String, crate::error::Error>> + Send + 'static,
{
    let mut config = objectiveai::ConfigClient::new(None::<String>).read()?;

    let api_mode = config.api().get_mode();
    let viewer_mode = config.viewer().get_mode();

    match (api_mode, viewer_mode) {
        #[cfg(feature = "viewer")]
        (objectiveai::ApiMode::Local, objectiveai::ViewerMode::Local) => {
            run_local_api_local_viewer(config, task).await
        }
        #[cfg(not(feature = "viewer"))]
        (objectiveai::ApiMode::Local, objectiveai::ViewerMode::Local) => {
            run_local_api_remote_viewer(config, task).await
        }
        (objectiveai::ApiMode::Local, objectiveai::ViewerMode::Remote) => {
            run_local_api_remote_viewer(config, task).await
        }
        #[cfg(feature = "viewer")]
        (objectiveai::ApiMode::Remote, objectiveai::ViewerMode::Local) => {
            run_remote_api_local_viewer(config, task).await
        }
        #[cfg(not(feature = "viewer"))]
        (objectiveai::ApiMode::Remote, objectiveai::ViewerMode::Local) => {
            run_remote_api_remote_viewer(config, task).await
        }
        (objectiveai::ApiMode::Remote, objectiveai::ViewerMode::Remote) => {
            run_remote_api_remote_viewer(config, task).await
        }
    }
    .map(crate::Output::Api)
}

// -- Variants --

/// Spawns both a local API server and a local Tauri viewer window.
/// The API's viewer client is pointed at the local viewer's bound address.
/// Viewer serve blocks the main thread; the task runs on a spawned tokio task
/// and kills the viewer via the exiter when it completes.
#[cfg(feature = "viewer")]
async fn run_local_api_local_viewer<F, Fut>(
    mut config: objectiveai::Config,
    task: F,
) -> Result<String, crate::error::Error>
where
    F: FnOnce(objectiveai::HttpClient) -> Fut + Send + 'static,
    Fut: Future<Output = Result<String, crate::error::Error>> + Send + 'static,
{
    let (viewer_config, secret_from_env, config_signature) = build_viewer_config(&mut config)?;

    // ENV mismatch check: VIEWER_SECRET and VIEWER_SIGNATURE must both or neither come from ENV
    let api_builder_peek = objectiveai_api::ConfigBuilder::init_from_env().unwrap_or_default();
    if secret_from_env != api_builder_peek.viewer_signature.is_some() {
        return Err(crate::error::Error::ViewerSecretSignatureEnvMismatch);
    }

    // Setup viewer first — we need its bound port for the API config
    let (viewer_listener, viewer_app, viewer_rx) = objectiveai_viewer::setup(viewer_config).await
        .map_err(crate::error::Error::ViewerSetup)?;
    let viewer_addr = viewer_listener.local_addr()
        .map_err(crate::error::Error::ViewerSetup)?;
    let viewer_addr_str = format!("http://127.0.0.1:{}", viewer_addr.port());

    // Setup API with viewer address + signature so its viewer client can POST events
    let api_config = build_api_config(&mut config, Some(viewer_addr_str.clone()), config_signature.clone());

    let (api_listener, api_router) = objectiveai_api::setup(api_config).await
        .map_err(crate::error::Error::ApiSetup)?;
    let api_addr = api_listener.local_addr()
        .map_err(crate::error::Error::ApiSetup)?;
    tokio::spawn(async move {
        let _ = objectiveai_api::serve(api_listener, api_router).await;
    });

    // HttpClient points at the local API, with viewer headers for event forwarding
    let http_client = build_http_client(
        &mut config,
        Some(format!("http://127.0.0.1:{}", api_addr.port())),
        Some(viewer_addr_str),
        config_signature,
    );

    // Exiter pattern: task runs on a spawned task, kills the viewer when done
    let (exiter_tx, exiter_rx) = tokio::sync::oneshot::channel::<objectiveai_viewer::Exiter>();
    let (result_tx, result_rx) = tokio::sync::oneshot::channel();

    tokio::spawn(async move {
        let result = task(http_client).await;
        result_tx.send(result).ok();
        let exiter = exiter_rx.await.unwrap();
        exiter(0);
    });

    // Blocks the main thread until the exiter is called
    let _ = objectiveai_viewer::serve(
        viewer_listener, viewer_app, viewer_rx, Some(exiter_tx),
    );

    result_rx.await.unwrap()
}

/// Spawns a local API server only. No viewer window — viewer address and
/// signature come from ENV/config headers (pointing at a remote viewer).
async fn run_local_api_remote_viewer<F, Fut>(
    mut config: objectiveai::Config,
    task: F,
) -> Result<String, crate::error::Error>
where
    F: FnOnce(objectiveai::HttpClient) -> Fut + Send + 'static,
    Fut: Future<Output = Result<String, crate::error::Error>> + Send + 'static,
{
    // Viewer fields overlay from headers (remote viewer configured externally)
    let api_config = build_api_config(&mut config, None, None);

    let (api_listener, api_router) = objectiveai_api::setup(api_config).await
        .map_err(crate::error::Error::ApiSetup)?;
    let api_addr = api_listener.local_addr()
        .map_err(crate::error::Error::ApiSetup)?;
    tokio::spawn(async move {
        let _ = objectiveai_api::serve(api_listener, api_router).await;
    });

    // HttpClient points at the local API; viewer fields from ENV/config
    let http_client = build_http_client(
        &mut config,
        Some(format!("http://127.0.0.1:{}", api_addr.port())),
        None,
        None,
    );

    task(http_client).await
}

/// Spawns a local Tauri viewer window only. The API is remote — the task's
/// HttpClient sends viewer headers so the remote API can forward events
/// to our local viewer.
#[cfg(feature = "viewer")]
async fn run_remote_api_local_viewer<F, Fut>(
    mut config: objectiveai::Config,
    task: F,
) -> Result<String, crate::error::Error>
where
    F: FnOnce(objectiveai::HttpClient) -> Fut + Send + 'static,
    Fut: Future<Output = Result<String, crate::error::Error>> + Send + 'static,
{
    let (viewer_config, _, config_signature) = build_viewer_config(&mut config)?;

    let (viewer_listener, viewer_app, viewer_rx) = objectiveai_viewer::setup(viewer_config).await
        .map_err(crate::error::Error::ViewerSetup)?;
    let viewer_addr = viewer_listener.local_addr()
        .map_err(crate::error::Error::ViewerSetup)?;

    // HttpClient points at the remote API, with local viewer address + signature
    let http_client = build_http_client(
        &mut config,
        None,
        Some(format!("http://127.0.0.1:{}", viewer_addr.port())),
        config_signature,
    );

    // Exiter pattern: task runs on a spawned task, kills the viewer when done
    let (exiter_tx, exiter_rx) = tokio::sync::oneshot::channel::<objectiveai_viewer::Exiter>();
    let (result_tx, result_rx) = tokio::sync::oneshot::channel();

    tokio::spawn(async move {
        let result = task(http_client).await;
        result_tx.send(result).ok();
        let exiter = exiter_rx.await.unwrap();
        exiter(0);
    });

    // Blocks the main thread until the exiter is called
    let _ = objectiveai_viewer::serve(
        viewer_listener, viewer_app, viewer_rx, Some(exiter_tx),
    );

    result_rx.await.unwrap()
}

/// No local spawning. The API and viewer are both remote.
/// HttpClient gets all values from ENV and config file.
async fn run_remote_api_remote_viewer<F, Fut>(
    mut config: objectiveai::Config,
    task: F,
) -> Result<String, crate::error::Error>
where
    F: FnOnce(objectiveai::HttpClient) -> Fut + Send + 'static,
    Fut: Future<Output = Result<String, crate::error::Error>> + Send + 'static,
{
    let http_client = build_http_client(&mut config, None, None, None);
    task(http_client).await
}

// -- Shared helpers --

/// Builds the local viewer config. Priority: ENV → config file → defaults.
///
/// Reads the secret/signature pair from `ViewerLocalConfig`. Errors if one is
/// set without the other. The secret goes to the viewer; the signature is
/// returned so the caller can forward it to the API and HttpClient.
///
/// Returns `(viewer_config, secret_from_env, config_signature)`.
/// `secret_from_env` lets the caller perform the ENV mismatch check against
/// the API builder's `VIEWER_SIGNATURE`.
#[cfg(feature = "viewer")]
fn build_viewer_config(
    config: &mut objectiveai::Config,
) -> Result<(objectiveai_viewer::Config, bool, Option<String>), crate::error::Error> {
    // Config file: both secret and signature must be present, or both absent
    let viewer_local = config.viewer().local();
    let (config_secret, config_signature) = match (viewer_local.get_secret(), viewer_local.get_signature()) {
        (Some(s), Some(sig)) => (Some(String::from(s)), Some(String::from(sig))),
        (None, None) => (None, None),
        _ => return Err(crate::error::Error::ViewerSecretSignatureConfigMismatch),
    };

    let mut builder = objectiveai_viewer::ConfigBuilder::init_from_env().unwrap_or_default();
    let secret_from_env = builder.secret.is_some();

    // Config file overlay: only fills if ENV didn't provide a secret
    if builder.secret.is_none() {
        builder.secret = config_secret;
    }

    // Force overrides: local viewer always binds to localhost on a system-assigned port
    builder.address = Some("127.0.0.1".to_string());
    builder.port = Some(0);
    builder.suppress_output = Some(true);

    Ok((builder.build(), secret_from_env, config_signature))
}

/// Builds the local API server config. Priority: ENV → config file → defaults.
///
/// Overlays authorization headers and `claude_agent_sdk` from the config file
/// for any fields not already set by ENV.
///
/// `viewer_address` and `viewer_signature` control how viewer fields are resolved:
/// - `Some(value)`: use this value (for local viewer — address is the bound port,
///   signature is from the config pair). Signature only sets if ENV didn't provide one.
/// - `None`: overlay from `ApiHeadersConfig` like all other fields (for remote viewer).
///
/// Always force-overrides `address`, `port`, and `suppress_output` for local binding.
fn build_api_config(
    config: &mut objectiveai::Config,
    viewer_address: Option<String>,
    viewer_signature: Option<String>,
) -> objectiveai_api::Config {
    let mut builder = objectiveai_api::ConfigBuilder::init_from_env().unwrap_or_default();

    // Config file overlay: read claude_agent_sdk before borrowing headers
    builder.claude_agent_sdk = builder.claude_agent_sdk.or(config.api().local().get_claude_agent_sdk());

    // Config file overlay: fill None fields from ApiHeadersConfig
    let headers = config.api().headers();
    builder.objectiveai_authorization = builder.objectiveai_authorization.or(headers.get_x_objectiveai_authorization().map(String::from));
    builder.openrouter_authorization = builder.openrouter_authorization.or(headers.get_x_openrouter_authorization().map(String::from));
    builder.github_authorization = builder.github_authorization.or(headers.get_x_github_authorization().map(String::from));
    if builder.mcp_authorization.is_none() {
        builder.mcp_authorization = headers.get_x_mcp_authorization()
            .and_then(|m| serde_json::to_string(m).ok());
    }
    builder.user_agent = builder.user_agent.or(headers.get_user_agent().map(String::from));
    builder.http_referer = builder.http_referer.or(headers.get_http_referer().map(String::from));
    builder.x_title = builder.x_title.or(headers.get_x_title().map(String::from));
    builder.commit_author_name = builder.commit_author_name.or(headers.get_x_commit_author_name().map(String::from));
    builder.commit_author_email = builder.commit_author_email.or(headers.get_x_commit_author_email().map(String::from));

    // Viewer fields: override for local viewer, overlay from headers for remote
    match viewer_address {
        Some(addr) => builder.viewer_address = Some(addr),
        None => builder.viewer_address = builder.viewer_address.or(headers.get_x_viewer_address().map(String::from)),
    }
    match viewer_signature {
        Some(sig) => {
            if builder.viewer_signature.is_none() {
                builder.viewer_signature = Some(sig);
            }
        }
        None => builder.viewer_signature = builder.viewer_signature.or(headers.get_x_viewer_signature().map(String::from)),
    }

    // Force overrides: local API always binds to localhost on a system-assigned port
    builder.address = Some("127.0.0.1".to_string());
    builder.port = Some(0);
    builder.suppress_output = Some(true);

    builder.build()
}

/// Builds the SDK HttpClient for the task closure. Priority: ENV → config file → defaults.
///
/// `HttpClient::new` with all-None params picks up ENV vars via the `env` feature.
/// Then config file headers fill any remaining None fields.
///
/// - `address`: `Some` for local API (bound address), `None` for remote (env/default).
/// - `viewer_address`: `Some` for local viewer (bound address), `None` for remote (normal overlay).
/// - `viewer_signature`: `Some` for local viewer (config pair signature), `None` for remote.
///   Only sets if ENV and config overlay didn't already provide one.
fn build_http_client(
    config: &mut objectiveai::Config,
    address: Option<String>,
    viewer_address: Option<String>,
    viewer_signature: Option<String>,
) -> objectiveai::HttpClient {
    // ENV fallbacks handled internally by HttpClient::new (env feature).
    // Viewer signature and address are passed directly so ENV takes priority
    // over the caller's values (HttpClient::new checks env first for None params,
    // but our params are Some when the caller provides overrides).
    let mut http_client = objectiveai::HttpClient::new(
        reqwest::Client::new(),
        address,          // local API bound addr, or None for remote (env/default)
        None::<String>,   // authorization
        None::<String>,   // user_agent
        None::<String>,   // x_title
        None::<String>,   // http_referer
        None::<String>,   // x_github_authorization
        None::<String>,   // x_openrouter_authorization
        None,             // x_mcp_authorization
        viewer_signature, // x_viewer_signature: local viewer pair, or None for remote
        viewer_address,   // x_viewer_address: local viewer bound addr, or None for remote
        None::<String>,   // x_commit_author_name
        None::<String>,   // x_commit_author_email
    );

    // Config file overlay: fill remaining None fields from ApiHeadersConfig
    let headers = config.api().headers();
    if http_client.authorization.is_none() {
        http_client.authorization = headers.get_x_objectiveai_authorization().map(|s| Arc::new(s.to_string()));
    }
    if http_client.user_agent.is_none() {
        http_client.user_agent = headers.get_user_agent().map(String::from);
    }
    if http_client.x_title.is_none() {
        http_client.x_title = headers.get_x_title().map(String::from);
    }
    if http_client.http_referer.is_none() {
        http_client.http_referer = headers.get_http_referer().map(String::from);
    }
    if http_client.x_github_authorization.is_none() {
        http_client.x_github_authorization = headers.get_x_github_authorization().map(|s| Arc::new(s.to_string()));
    }
    if http_client.x_openrouter_authorization.is_none() {
        http_client.x_openrouter_authorization = headers.get_x_openrouter_authorization().map(|s| Arc::new(s.to_string()));
    }
    if http_client.x_mcp_authorization.is_none() {
        http_client.x_mcp_authorization = headers.get_x_mcp_authorization()
            .map(|m| Arc::new(m.iter().map(|(k, v)| (k.clone(), v.clone())).collect()));
    }
    if http_client.x_viewer_signature.is_none() {
        http_client.x_viewer_signature = headers.get_x_viewer_signature().map(|s| Arc::new(s.to_string()));
    }
    if http_client.x_viewer_address.is_none() {
        http_client.x_viewer_address = headers.get_x_viewer_address().map(|s| Arc::new(s.to_string()));
    }
    if http_client.x_commit_author_name.is_none() {
        http_client.x_commit_author_name = headers.get_x_commit_author_name().map(|s| Arc::new(s.to_string()));
    }
    if http_client.x_commit_author_email.is_none() {
        http_client.x_commit_author_email = headers.get_x_commit_author_email().map(|s| Arc::new(s.to_string()));
    }

    http_client
}
