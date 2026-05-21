//! `tools` subcommand tree — list / get / install local-filesystem tools.
//!
//! Mirrors `crate::plugins`'s built-in listing surface but without the
//! GitHub install pipeline. Tools are hand-placed under
//! `~/.objectiveai/tools/`; `tools install` only prints authoring
//! instructions.

use clap::Subcommand;
use objectiveai_sdk::cli::output::{Handle, Notification, Output, Tool, Tools};

mod install;

#[derive(Subcommand)]
pub enum Commands {
    /// Get a single tool's manifest by name. Emits the manifest as
    /// `{"tool": <manifest>}` when found, or `{"tool": null}` when
    /// the manifest file is missing / unreadable / malformed (same
    /// silent-skip policy as `list`).
    Get {
        /// Tool name (filename stem of the manifest in
        /// `~/.objectiveai/tools/`).
        name: String,
    },
    /// Get instructions for authoring a tool in your local
    /// `~/.objectiveai/tools/` directory by hand. Takes no args —
    /// the CLI prints an INSTRUCTIONS.md telling the agent the
    /// manifest schema command path and the layout convention.
    /// Nothing is installed.
    Install,
    /// List installed tools (every `.json` manifest in
    /// `~/.objectiveai/tools/`). Sorted by manifest mtime, most
    /// recent first. Supports `--offset` / `--limit` for pagination.
    List {
        #[arg(long, default_value_t = 0)]
        offset: usize,
        #[arg(long, default_value_t = 100)]
        limit: usize,
    },
}

impl Commands {
    pub async fn handle(
        self,
        cli_config: &crate::Config,
        handle: &Handle,
    ) -> Result<(), crate::error::Error> {
        match self {
            Commands::Get { name } => get(cli_config, handle, &name).await,
            Commands::Install => install::emit_instructions(handle).await,
            Commands::List { offset, limit } => list(cli_config, handle, offset, limit).await,
        }
    }
}

async fn get(
    cli_config: &crate::Config,
    handle: &Handle,
    name: &str,
) -> Result<(), crate::error::Error> {
    let fs_client = objectiveai_sdk::filesystem::Client::new(
        cli_config.config_base_dir.as_deref(),
        cli_config.commit_author_name.as_deref(),
        cli_config.commit_author_email.as_deref(),
    );
    let tool = fs_client.get_tool(name).await;
    Output::<Tool>::Notification(Notification { agent_id: None, value: Tool { tool } })
        .emit(handle)
        .await;
    Ok(())
}

async fn list(
    cli_config: &crate::Config,
    handle: &Handle,
    offset: usize,
    limit: usize,
) -> Result<(), crate::error::Error> {
    let fs_client = objectiveai_sdk::filesystem::Client::new(
        cli_config.config_base_dir.as_deref(),
        cli_config.commit_author_name.as_deref(),
        cli_config.commit_author_email.as_deref(),
    );
    let tools = fs_client.list_tools(offset, limit).await;
    Output::<Tools>::Notification(Notification { agent_id: None, value: Tools { tools } })
        .emit(handle)
        .await;
    Ok(())
}
