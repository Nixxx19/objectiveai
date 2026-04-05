pub mod config;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Favorites configuration
    Config {
        #[command(subcommand)]
        command: config::Commands,
    },
}

impl Commands {
    pub fn handle(self, cli_config: &crate::Config) -> Result<crate::Output, crate::error::Error> {
        match self {
            Commands::Config { command } => command.handle(cli_config),
        }
    }
}
