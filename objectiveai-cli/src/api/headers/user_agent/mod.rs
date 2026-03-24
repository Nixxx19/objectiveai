pub mod config;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    Config { #[command(subcommand)] command: config::Commands },
}

impl Commands {
    pub fn handle(self) -> Result<crate::Output, crate::error::Error> {
        match self { Commands::Config { command } => command.handle() }
    }
}
