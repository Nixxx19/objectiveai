#[tokio::main]
async fn main() {
    let _ = dotenv::dotenv();
    // Collect argv once so the updater can forward it to the re-exec'd
    // new binary while run() still gets the same sequence.
    let args: Vec<std::ffi::OsString> = std::env::args_os().collect();
    // Best-effort auto-update. No-op unless the `updater` feature is on;
    // may never return because the replacement has been spawned with
    // the same argv. Any error inside is logged to stderr and swallowed.
    objectiveai_cli::update::maybe_auto_update(args.clone()).await;
    match objectiveai_cli::run(args).await {
        Ok(output) => println!("{output}"),
        Err(e) => {
            println!("error: {e}");
            std::process::exit(1);
        }
    }
}
