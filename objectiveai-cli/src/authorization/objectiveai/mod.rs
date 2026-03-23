pub mod config;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// ObjectiveAI key configuration
    Config {
        #[command(subcommand)]
        command: config::Commands,
    },
}

impl Commands {
    pub fn handle(self) -> Result<Option<String>, crate::error::Error> {
        match self {
            Commands::Config { command } => command.handle(),
        }
    }
}
