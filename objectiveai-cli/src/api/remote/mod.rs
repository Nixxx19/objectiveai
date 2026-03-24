pub mod config;
pub mod objectiveai_address;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Remote API configuration
    Config {
        #[command(subcommand)]
        command: config::Commands,
    },
    /// ObjectiveAI address
    ObjectiveaiAddress {
        #[command(subcommand)]
        command: objectiveai_address::Commands,
    },
}

impl Commands {
    pub fn handle(self) -> Result<crate::Output, crate::error::Error> {
        match self {
            Commands::Config { command } => command.handle(),
            Commands::ObjectiveaiAddress { command } => command.handle(),
        }
    }
}
