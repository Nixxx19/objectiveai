#[tokio::main]
async fn main() {
    let _ = dotenv::dotenv();
    let cli_config = objectiveai_cli::load_config();
    let args: Vec<std::ffi::OsString> = std::env::args_os().collect();
    // Default destination: this process's stdout. Programmatic embedders
    // constructing their own `cli::run` call can supply
    // `Handle::Stdin(Arc::new(Mutex::new(child.stdin.take().unwrap())))`
    // or `Handle::Collect(_)` instead.
    let handle: objectiveai_sdk::cli::output::Handle =
        objectiveai_sdk::cli::output::Handle::Stdout;
    let code = objectiveai_cli::run(args, &cli_config, handle).await;
    std::process::exit(code);
}
