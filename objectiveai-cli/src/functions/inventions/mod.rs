pub mod config;
pub mod remote;
pub mod recursive;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Recursive function invention
    Recursive {
        #[command(subcommand)]
        command: recursive::Commands,
    },
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
    pub async fn handle(self) -> Result<crate::Output, crate::error::Error> {
        match self {
            Commands::Recursive { command } => command.handle().await,
            Commands::Config { command } => command.handle(),
            Commands::Remote { command } => command.handle(),
        }
    }
}
