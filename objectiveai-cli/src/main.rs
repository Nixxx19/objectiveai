#[tokio::main]
async fn main() {
    let _ = dotenv::dotenv();
    let mut cli_config = objectiveai_cli::load_config();
    // No env-supplied agent_id? Stamp `"cli"` so every outbound
    // `X-OBJECTIVEAI-AGENT-ID` header — direct HTTP from
    // `build_http_client`, and forwarded `--objectiveai-agent-id` to
    // `objectiveai-cli-stream` — identifies the request as
    // originating from the cli binary. The api server then mints
    // composite ids as `cli/<local-id>` for every agent spawned from
    // here. Scoped to the cli binary's entry point: programmatic
    // callers of `objectiveai_cli::load_config()` (notably
    // `objectiveai-mcp`) keep `None` here so the per-request
    // header-stamp flow stays parent-less when no inbound header.
    cli_config.agent_id.get_or_insert_with(|| "cli".to_string());
    let args: Vec<std::ffi::OsString> = std::env::args_os().collect();
    // Default destination: this process's stdout. Programmatic embedders
    // constructing their own `cli::run` call can supply a `Handle` whose
    // `destination` is `HandleDestination::Stdin(...)` or
    // `HandleDestination::Collect(_)` instead.
    //
    // Stamp the handle's agent_id from cli_config so every emitted
    // notification + error line carries `X-OBJECTIVEAI-AGENT-ID`'s
    // value (env `OBJECTIVEAI_AGENT_ID`, or `"cli"` per the default
    // above). Per-request callers like objectiveai-mcp build their
    // own Handle and set this from the `X-OBJECTIVEAI-AGENT-ID`
    // request header instead.
    let mut handle = objectiveai_sdk::cli::output::Handle::stdout();
    handle.agent_id = cli_config.agent_id.clone();
    let code = objectiveai_cli::run(args, &cli_config, handle).await;
    std::process::exit(code);
}
