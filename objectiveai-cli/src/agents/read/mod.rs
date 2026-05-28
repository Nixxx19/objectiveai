pub mod pending;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Drain pending queue items for the given spawned agent id(s)
    Pending(pending::CommandArgs),
}

impl Commands {
    pub async fn handle(self, cli_config: &crate::Config, handle: &objectiveai_sdk::cli::output::Handle) -> Result<(), crate::error::Error> {
        match self {
            Commands::Pending(args) => pending::handle(args, cli_config, handle).await,
        }
    }
}
