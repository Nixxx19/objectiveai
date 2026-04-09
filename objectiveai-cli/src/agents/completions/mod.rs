pub mod create;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Create an agent completion
    Create {
        #[command(subcommand)]
        command: create::Commands,
    },
}

impl Commands {
    pub async fn handle(self, cli_config: &crate::Config) -> Result<crate::Output, crate::error::Error> {
        match self {
            Commands::Create { command } => command.handle(cli_config).await,
        }
    }
}
