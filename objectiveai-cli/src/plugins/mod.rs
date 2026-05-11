//! Plugins management + external plugin dispatcher.
//!
//! Two responsibilities live here:
//!
//! 1. The built-in `plugins` subcommand tree ([`Commands`]) — currently
//!    just `plugins list`, which enumerates every manifest in
//!    `~/.objectiveai/plugins/` via [`objectiveai::filesystem::Client::list_plugins`].
//! 2. The catch-all dispatcher ([`dispatch_external`]) invoked by clap's
//!    `external_subcommand` for any unknown top-level subcommand.
//!
//! For (2): when `objectiveai <name> <args…>` is invoked and `<name>`
//! isn't a built-in subcommand, clap's `external_subcommand` mechanism
//! routes the call to [`dispatch_external`]. We resolve `<name>` against
//! `~/.objectiveai/plugins/`, spawn the binary with `<args…>`, and
//! consume its stdout as a JSONL stream of [`PluginOutput`]. Per-line
//! dispatch:
//!
//! - `Error` → forward via [`Output::Error`]
//! - `Notification(Value)` → forward via [`Output::Notification`]
//! - `Command { command }` → tokenize the string and spawn a recursive
//!   `cli::run` call (fire-and-forget; multiple in flight is fine)
//! - parse failure → re-emit the raw line as a string-valued
//!   notification, so it still appears in the host's JSONL stream
//!
//! Plugin stderr is forwarded raw to this process's own stderr (same
//! pattern as `api::detach`). The plugin's exit code becomes this
//! function's `Err(PluginExit)` — which `run()` then emits as a fatal
//! error and converts to exit code 1.

use clap::Subcommand;
use objectiveai_cli_lib::output::{Handle, Notification, Output, Plugins};
use objectiveai_cli_lib::plugins::PluginOutput;
use tokio::io::AsyncBufReadExt;
use tokio::task::JoinHandle;

#[derive(Subcommand)]
pub enum Commands {
    /// List installed plugins (every `.json` manifest in
    /// `~/.objectiveai/plugins/`).
    List,
}

impl Commands {
    pub async fn handle(
        self,
        cli_config: &crate::Config,
        handle: &Handle,
    ) -> Result<(), crate::error::Error> {
        match self {
            Commands::List => list(cli_config, handle).await,
        }
    }
}

async fn list(
    cli_config: &crate::Config,
    handle: &Handle,
) -> Result<(), crate::error::Error> {
    let fs_client = objectiveai::filesystem::Client::new(
        cli_config.config_base_dir.as_deref(),
        cli_config.commit_author_name.as_deref(),
        cli_config.commit_author_email.as_deref(),
    );
    let plugins = fs_client.list_plugins().await;
    Output::<Plugins>::Notification(Notification { value: Plugins { plugins } })
        .emit(handle)
        .await;
    Ok(())
}

pub async fn dispatch_external(
    args: Vec<String>,
    cli_config: &crate::Config,
    handle: &Handle,
) -> Result<(), crate::error::Error> {
    let mut iter = args.into_iter();
    let name_str = iter
        .next()
        .ok_or(crate::error::Error::MissingArgs("plugin name"))?;
    let rest: Vec<String> = iter.collect();

    let fs_client = objectiveai::filesystem::Client::new(
        cli_config.config_base_dir.as_deref(),
        cli_config.commit_author_name.as_deref(),
        cli_config.commit_author_email.as_deref(),
    );
    let exe = match fs_client.resolve_plugin(&name_str).await {
        Some(p) => p,
        None => return Err(crate::error::Error::PluginNotFound(name_str)),
    };

    let mut child = tokio::process::Command::new(&exe)
        .args(&rest)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(crate::error::Error::PluginSpawn)?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let stderr_task = tokio::spawn(forward_stderr(stderr));

    let mut command_tasks: Vec<JoinHandle<i32>> = Vec::new();
    let mut reader = tokio::io::BufReader::new(stdout);
    let mut line = String::new();
    loop {
        line.clear();
        let n = reader
            .read_line(&mut line)
            .await
            .map_err(crate::error::Error::PluginRead)?;
        if n == 0 {
            break;
        }
        let trimmed = line.trim_end_matches(['\r', '\n']);
        match serde_json::from_str::<PluginOutput>(trimmed) {
            Ok(PluginOutput::Error(e)) => {
                Output::<serde_json::Value>::Error(e).emit(handle).await;
            }
            Ok(PluginOutput::Notification(value)) => {
                Output::<serde_json::Value>::Notification(Notification { value })
                    .emit(handle)
                    .await;
            }
            Ok(PluginOutput::Command { command }) => {
                command_tasks.push(spawn_command(command, cli_config, handle));
            }
            Err(_) => {
                let value = serde_json::Value::String(trimmed.to_string());
                Output::<serde_json::Value>::Notification(Notification { value })
                    .emit(handle)
                    .await;
            }
        }
    }

    // Drain any in-flight Command runs the plugin queued before exiting.
    for t in command_tasks {
        let _ = t.await;
    }
    let _ = stderr_task.await;

    let status = child
        .wait()
        .await
        .map_err(crate::error::Error::PluginRead)?;
    if status.success() {
        Ok(())
    } else {
        Err(crate::error::Error::PluginExit(status.code().unwrap_or(1)))
    }
}

/// Tokenize the plugin's `command` string and spawn `cli::run` on it.
/// Uses whitespace splitting — quoted args aren't supported (upgrade
/// to `shlex` if a real plugin needs them).
fn spawn_command(
    command: String,
    cli_config: &crate::Config,
    handle: &Handle,
) -> JoinHandle<i32> {
    let tokens: Vec<String> = command.split_whitespace().map(String::from).collect();
    // `run()` expects argv[0] to be the binary name (clap ignores its
    // content). Prepend a placeholder.
    let mut argv: Vec<String> = Vec::with_capacity(tokens.len() + 1);
    argv.push(String::from("objectiveai"));
    argv.extend(tokens);

    let cfg = cli_config.clone();
    let handle = handle.clone();
    tokio::spawn(async move { crate::run::run(argv, &cfg, handle).await })
}

async fn forward_stderr(mut stderr: tokio::process::ChildStderr) {
    use tokio::io::AsyncReadExt;
    let mut buf = [0u8; 4096];
    loop {
        match stderr.read(&mut buf).await {
            Ok(0) | Err(_) => return,
            Ok(n) => {
                use std::io::Write;
                let _ = std::io::stderr().write_all(&buf[..n]);
            }
        }
    }
}
