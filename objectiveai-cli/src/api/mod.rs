pub mod config;
pub mod address;
pub mod port;
pub mod claude_agent_sdk;
pub mod codex_sdk;
pub mod headers;
mod run;
pub mod detach;

pub use run::*;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// API configuration
    Config {
        #[command(subcommand)]
        command: config::Commands,
    },
    /// Local API server bind address
    Address {
        #[command(subcommand)]
        command: address::Commands,
    },
    /// Local API server bind port
    Port {
        #[command(subcommand)]
        command: port::Commands,
    },
    /// Claude Agent SDK enabled
    ClaudeAgentSdk {
        #[command(subcommand)]
        command: claude_agent_sdk::Commands,
    },
    /// Codex SDK enabled
    CodexSdk {
        #[command(subcommand)]
        command: codex_sdk::Commands,
    },
    /// API headers configuration
    Headers {
        #[command(subcommand)]
        command: headers::Commands,
    },
}

impl Commands {
    pub async fn handle(self, cli_config: &crate::Config, handle: &objectiveai_sdk::cli::output::Handle) -> Result<(), crate::error::Error> {
        match self {
            Commands::Config { command } => command.handle(cli_config, handle).await,
            Commands::Address { command } => command.handle(cli_config, handle).await,
            Commands::Port { command } => command.handle(cli_config, handle).await,
            Commands::ClaudeAgentSdk { command } => command.handle(cli_config, handle).await,
            Commands::CodexSdk { command } => command.handle(cli_config, handle).await,
            Commands::Headers { command } => command.handle(cli_config, handle).await,
        }
    }
}
