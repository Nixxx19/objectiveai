#[tokio::main]
async fn main() {
    let _ = dotenv::dotenv();
    // Build the top-level config ONCE after `.env` is loaded, so the
    // auto-updater and `run` share the same env-sourced view (notably
    // `GITHUB_AUTHORIZATION`, used by the updater).
    let cli_config = objectiveai_cli::load_config();
    // Collect argv once so the updater can forward it to the re-exec'd
    // new binary while run() still gets the same sequence.
    let args: Vec<std::ffi::OsString> = std::env::args_os().collect();
    // No subprocess emission by default — emit goes to stdout/stderr.
    // Programmatic embedders constructing their own `cli::run` call can
    // supply `Some(Arc::new(Mutex::new(child.stdin.take().unwrap())))`.
    let handle: objectiveai_cli_lib::output::Handle = None;
    // Best-effort auto-update. No-op unless the `updater` feature is on;
    // may never return because the replacement has been spawned with
    // the same argv. Any error inside is emitted as a non-fatal warn-level
    // notification and swallowed.
    objectiveai_cli::update::maybe_auto_update(args.clone(), &cli_config, &handle).await;
    if let Err(err) = objectiveai_cli::run(args, &cli_config, handle.clone()).await {
        objectiveai_cli_lib::output::Output::<serde_json::Value>::Error(err)
            .emit(&handle)
            .await;
        std::process::exit(1);
    }
}
