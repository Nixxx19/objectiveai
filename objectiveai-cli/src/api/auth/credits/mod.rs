pub mod get;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    Get,
}

impl Commands {
    pub async fn handle(self, cli_config: &crate::Config, handle: &objectiveai_sdk::cli::output::Handle) -> Result<(), crate::error::Error> {
        match self {
            Commands::Get => get::handle(cli_config, handle).await,
        }
    }
}
