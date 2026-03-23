pub mod config;
pub mod objectiveai;
pub mod openrouter;
pub mod github;
pub mod mcp;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Authorization configuration
    Config {
        #[command(subcommand)]
        command: config::Commands,
    },
    /// ObjectiveAI API key
    Objectiveai {
        #[command(subcommand)]
        command: objectiveai::Commands,
    },
    /// OpenRouter API key
    Openrouter {
        #[command(subcommand)]
        command: openrouter::Commands,
    },
    /// GitHub token
    Github {
        #[command(subcommand)]
        command: github::Commands,
    },
    /// MCP authorization entries
    Mcp {
        #[command(subcommand)]
        command: mcp::Commands,
    },
}

impl Commands {
    pub fn handle(self) -> Result<crate::Output, crate::error::Error> {
        match self {
            Commands::Config { command } => command.handle(),
            Commands::Objectiveai { command } => command.handle(),
            Commands::Openrouter { command } => command.handle(),
            Commands::Github { command } => command.handle(),
            Commands::Mcp { command } => command.handle(),
        }
    }
}
