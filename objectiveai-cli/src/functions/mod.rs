pub mod config;
pub mod favorites;
pub mod inventions;
pub mod profiles;
pub mod executions;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Functions configuration
    Config {
        #[command(subcommand)]
        command: config::Commands,
    },
    /// Manage function favorites
    Favorites {
        #[command(subcommand)]
        command: favorites::Commands,
    },
    /// Functions inventions
    Inventions {
        #[command(subcommand)]
        command: inventions::Commands,
    },
    /// Functions profiles
    Profiles {
        #[command(subcommand)]
        command: profiles::Commands,
    },
}

impl Commands {
    pub fn handle(self) -> Result<Option<String>, crate::error::Error> {
        match self {
            Commands::Config { command } => command.handle(),
            Commands::Favorites { command } => command.handle(),
            Commands::Inventions { command } => command.handle(),
            Commands::Profiles { command } => command.handle(),
        }
    }
}
