pub mod create;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Create a recursive function invention
    Create {
        #[command(subcommand)]
        command: create::Commands,
    },
}

impl Commands {
    pub async fn handle(self) -> Result<crate::Output, crate::error::Error> {
        match self {
            Commands::Create { command } => command.handle().await,
        }
    }
}
