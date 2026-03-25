use std::future::Future;

/// Runs the API server and viewer, executes the provided async task,
/// then shuts down the viewer and returns the task's result.
pub async fn run<F, Fut>(task: F) -> Result<String, crate::error::Error>
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<String, crate::error::Error>> + Send + 'static,
{
    // Set up viewer and API server concurrently
    let viewer_config = objectiveai_viewer::ConfigBuilder::default().build();
    let api_config = objectiveai_api::ConfigBuilder::default().build();

    let (viewer_result, api_result) = tokio::join!(
        objectiveai_viewer::setup(viewer_config),
        setup_and_spawn_api(api_config),
    );

    let (viewer_listener, viewer_app, viewer_rx) = viewer_result
        .map_err(crate::error::Error::ViewerSetup)?;

    api_result?;

    // Create channel for receiving the Exiter from viewer serve
    let (exiter_tx, exiter_rx) = tokio::sync::oneshot::channel::<objectiveai_viewer::Exiter>();

    // Create channel for receiving the task result
    let (result_tx, result_rx) = tokio::sync::oneshot::channel();

    // Spawn the task — once it completes, kill the viewer
    tokio::spawn(async move {
        let result = task().await;
        result_tx.send(result).ok();

        // Wait for the exiter and exit Tauri
        let exiter = exiter_rx.await.unwrap();
        exiter(0);
    });

    // Run viewer serve on the main thread (blocks until Tauri exits)
    let _viewer_exit_code = objectiveai_viewer::serve(
        viewer_listener,
        viewer_app,
        viewer_rx,
        Some(exiter_tx),
    );

    // Read the task result
    result_rx.await.unwrap()
}

/// Sets up the API server and spawns serve on a tokio task.
async fn setup_and_spawn_api(config: objectiveai_api::Config) -> Result<(), crate::error::Error> {
    let (listener, router) = objectiveai_api::setup(config).await
        .map_err(crate::error::Error::ApiSetup)?;

    tokio::spawn(async move {
        let _ = objectiveai_api::serve(listener, router).await;
    });

    Ok(())
}
