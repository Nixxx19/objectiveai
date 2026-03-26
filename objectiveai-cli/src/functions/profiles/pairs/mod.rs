pub mod config;
pub mod favorites;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Pairs configuration
    Config {
        #[command(subcommand)]
        command: config::Commands,
    },
    /// Manage pair favorites
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
