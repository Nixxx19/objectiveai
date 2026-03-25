use std::future::Future;
use envconfig::Envconfig;

/// Reads config and dispatches to the appropriate run variant
/// based on API mode (Local/Remote) and Viewer mode (Local/Remote).
pub async fn run<F, Fut>(task: F) -> Result<crate::Output, crate::error::Error>
where
    F: FnOnce() -> Fut + Send + 'static,
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
            run_remote_api_remote_viewer(task).await
        }
        (objectiveai::ApiMode::Remote, objectiveai::ViewerMode::Remote) => {
            run_remote_api_remote_viewer(task).await
        }
    }
    .map(crate::Output::Api)
}

#[cfg(feature = "viewer")]
async fn run_local_api_local_viewer<F, Fut>(
    mut config: objectiveai::Config,
    task: F,
) -> Result<String, crate::error::Error>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<String, crate::error::Error>> + Send + 'static,
{
    // Read secret/signature from config — error if one is set without the other
    let viewer_local = config.viewer().local();
    let (config_secret, config_signature) = match (viewer_local.get_secret(), viewer_local.get_signature()) {
        (Some(s), Some(sig)) => (Some(String::from(s)), Some(String::from(sig))),
        (None, None) => (None, None),
        _ => return Err(crate::error::Error::ViewerSecretSignatureConfigMismatch),
    };

    // Init both builders from ENV
    let mut viewer_builder = objectiveai_viewer::ConfigBuilder::init_from_env().unwrap_or_default();
    let mut api_builder = objectiveai_api::ConfigBuilder::init_from_env().unwrap_or_default();

    // ENV mismatch check: viewer secret and API viewer_signature must both or neither come from ENV
    let viewer_secret_from_env = viewer_builder.secret.is_some();
    let api_viewer_signature_from_env = api_builder.viewer_signature.is_some();
    if viewer_secret_from_env != api_viewer_signature_from_env {
        return Err(crate::error::Error::ViewerSecretSignatureEnvMismatch);
    }

    // Viewer config: ENV → config file overlay → force overrides
    if viewer_builder.secret.is_none() {
        viewer_builder.secret = config_secret;
    }
    viewer_builder.address = Some("127.0.0.1".to_string());
    viewer_builder.port = Some(0);
    viewer_builder.suppress_output = Some(true);

    let viewer_config = viewer_builder.build();

    let (viewer_listener, viewer_app, viewer_rx) = objectiveai_viewer::setup(viewer_config).await
        .map_err(crate::error::Error::ViewerSetup)?;
    let viewer_addr = viewer_listener.local_addr()
        .map_err(crate::error::Error::ViewerSetup)?;

    // API config: ENV → config file overlay → force overrides
    let headers = config.api().headers();
    api_builder.objectiveai_authorization = api_builder.objectiveai_authorization.or(headers.get_x_objectiveai_authorization().map(String::from));
    api_builder.openrouter_authorization = api_builder.openrouter_authorization.or(headers.get_x_openrouter_authorization().map(String::from));
    api_builder.github_authorization = api_builder.github_authorization.or(headers.get_x_github_authorization().map(String::from));
    if api_builder.mcp_authorization.is_none() {
        api_builder.mcp_authorization = headers.get_x_mcp_authorization()
            .and_then(|m| serde_json::to_string(m).ok());
    }
    if api_builder.viewer_signature.is_none() {
        api_builder.viewer_signature = config_signature;
    }
    api_builder.user_agent = api_builder.user_agent.or(headers.get_user_agent().map(String::from));
    api_builder.http_referer = api_builder.http_referer.or(headers.get_http_referer().map(String::from));
    api_builder.x_title = api_builder.x_title.or(headers.get_x_title().map(String::from));
    api_builder.commit_author_name = api_builder.commit_author_name.or(headers.get_x_commit_author_name().map(String::from));
    api_builder.commit_author_email = api_builder.commit_author_email.or(headers.get_x_commit_author_email().map(String::from));
    api_builder.claude_agent_sdk = api_builder.claude_agent_sdk.or(config.api().local().get_claude_agent_sdk());

    api_builder.address = Some("127.0.0.1".to_string());
    api_builder.port = Some(0);
    api_builder.suppress_output = Some(true);
    api_builder.viewer_address = Some(format!("http://127.0.0.1:{}", viewer_addr.port()));

    let api_config = api_builder.build();

    let (api_listener, api_router) = objectiveai_api::setup(api_config).await
        .map_err(crate::error::Error::ApiSetup)?;
    tokio::spawn(async move {
        let _ = objectiveai_api::serve(api_listener, api_router).await;
    });

    // Exiter pattern: task completes → kills viewer
    let (exiter_tx, exiter_rx) = tokio::sync::oneshot::channel::<objectiveai_viewer::Exiter>();
    let (result_tx, result_rx) = tokio::sync::oneshot::channel();

    tokio::spawn(async move {
        let result = task().await;
        result_tx.send(result).ok();
        let exiter = exiter_rx.await.unwrap();
        exiter(0);
    });

    // Viewer serve blocks the main thread until exiter is called
    let _ = objectiveai_viewer::serve(
        viewer_listener, viewer_app, viewer_rx, Some(exiter_tx),
    );

    result_rx.await.unwrap()
}

async fn run_local_api_remote_viewer<F, Fut>(
    mut config: objectiveai::Config,
    task: F,
) -> Result<String, crate::error::Error>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<String, crate::error::Error>> + Send + 'static,
{
    let mut api_builder = objectiveai_api::ConfigBuilder::init_from_env().unwrap_or_default();

    // Config file overlay (only fills None fields)
    let headers = config.api().headers();
    api_builder.objectiveai_authorization = api_builder.objectiveai_authorization.or(headers.get_x_objectiveai_authorization().map(String::from));
    api_builder.openrouter_authorization = api_builder.openrouter_authorization.or(headers.get_x_openrouter_authorization().map(String::from));
    api_builder.github_authorization = api_builder.github_authorization.or(headers.get_x_github_authorization().map(String::from));
    if api_builder.mcp_authorization.is_none() {
        api_builder.mcp_authorization = headers.get_x_mcp_authorization()
            .and_then(|m| serde_json::to_string(m).ok());
    }
    api_builder.viewer_signature = api_builder.viewer_signature.or(headers.get_x_viewer_signature().map(String::from));
    api_builder.viewer_address = api_builder.viewer_address.or(headers.get_x_viewer_address().map(String::from));
    api_builder.user_agent = api_builder.user_agent.or(headers.get_user_agent().map(String::from));
    api_builder.http_referer = api_builder.http_referer.or(headers.get_http_referer().map(String::from));
    api_builder.x_title = api_builder.x_title.or(headers.get_x_title().map(String::from));
    api_builder.commit_author_name = api_builder.commit_author_name.or(headers.get_x_commit_author_name().map(String::from));
    api_builder.commit_author_email = api_builder.commit_author_email.or(headers.get_x_commit_author_email().map(String::from));
    api_builder.claude_agent_sdk = api_builder.claude_agent_sdk.or(config.api().local().get_claude_agent_sdk());

    // Force overrides
    api_builder.address = Some("127.0.0.1".to_string());
    api_builder.port = Some(0);
    api_builder.suppress_output = Some(true);

    let api_config = api_builder.build();

    let (api_listener, api_router) = objectiveai_api::setup(api_config).await
        .map_err(crate::error::Error::ApiSetup)?;
    tokio::spawn(async move {
        let _ = objectiveai_api::serve(api_listener, api_router).await;
    });

    task().await
}

#[cfg(feature = "viewer")]
async fn run_remote_api_local_viewer<F, Fut>(
    mut config: objectiveai::Config,
    task: F,
) -> Result<String, crate::error::Error>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<String, crate::error::Error>> + Send + 'static,
{
    // Read secret from config — error if one is set without the other
    let viewer_local = config.viewer().local();
    let config_secret = match (viewer_local.get_secret(), viewer_local.get_signature()) {
        (Some(s), Some(_)) => Some(String::from(s)),
        (None, None) => None,
        _ => return Err(crate::error::Error::ViewerSecretSignatureConfigMismatch),
    };

    // Viewer config: ENV → config file overlay → force overrides
    let mut viewer_builder = objectiveai_viewer::ConfigBuilder::init_from_env().unwrap_or_default();

    if viewer_builder.secret.is_none() {
        viewer_builder.secret = config_secret;
    }
    viewer_builder.address = Some("127.0.0.1".to_string());
    viewer_builder.port = Some(0);
    viewer_builder.suppress_output = Some(true);

    let viewer_config = viewer_builder.build();

    let (viewer_listener, viewer_app, viewer_rx) = objectiveai_viewer::setup(viewer_config).await
        .map_err(crate::error::Error::ViewerSetup)?;

    // Exiter pattern: task completes → kills viewer
    let (exiter_tx, exiter_rx) = tokio::sync::oneshot::channel::<objectiveai_viewer::Exiter>();
    let (result_tx, result_rx) = tokio::sync::oneshot::channel();

    tokio::spawn(async move {
        let result = task().await;
        result_tx.send(result).ok();
        let exiter = exiter_rx.await.unwrap();
        exiter(0);
    });

    // Viewer serve blocks the main thread until exiter is called
    let _ = objectiveai_viewer::serve(
        viewer_listener, viewer_app, viewer_rx, Some(exiter_tx),
    );

    result_rx.await.unwrap()
}

async fn run_remote_api_remote_viewer<F, Fut>(
    task: F,
) -> Result<String, crate::error::Error>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<String, crate::error::Error>> + Send + 'static,
{
    task().await
}
