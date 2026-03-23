pub mod config;
pub mod remote;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Inventions configuration
    Config {
        #[command(subcommand)]
        command: config::Commands,
    },
    /// Inventions remote
    Remote {
        #[command(subcommand)]
        command: remote::Commands,
    },
}

impl Commands {
    pub fn handle(self) -> Result<crate::Output, crate::error::Error> {
        match self {
            Commands::Config { command } => command.handle(),
            Commands::Remote { command } => command.handle(),
        }
    }
}
