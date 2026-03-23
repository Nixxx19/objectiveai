pub mod config;
pub mod favorites;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Swarms configuration
    Config {
        #[command(subcommand)]
        command: config::Commands,
    },
    /// Manage swarm favorites
    Favorites {
        #[command(subcommand)]
        command: favorites::Commands,
    },
}

impl Commands {
    pub fn handle(self) -> Result<crate::Output, crate::error::Error> {
        match self {
            Commands::Config { command } => command.handle(),
            Commands::Favorites { command } => command.handle(),
        }
    }
}
