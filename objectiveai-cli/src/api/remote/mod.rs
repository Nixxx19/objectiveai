pub mod config;
pub mod objectiveai_api_base;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Remote API configuration
    Config {
        #[command(subcommand)]
        command: config::Commands,
    },
    /// ObjectiveAI API base URL
    ObjectiveaiApiBase {
        #[command(subcommand)]
        command: objectiveai_api_base::Commands,
    },
}

impl Commands {
    pub fn handle(self) -> Result<crate::Output, crate::error::Error> {
        match self {
            Commands::Config { command } => command.handle(),
            Commands::ObjectiveaiApiBase { command } => command.handle(),
        }
    }
}
