pub mod all;
pub mod pending;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Return every queue item for the given spawned agent id(s).
    /// Still advances the pending watermark to the highest index,
    /// so a subsequent `read pending` returns empty until new rows
    /// land.
    All(all::CommandArgs),
    /// Drain pending queue items (past the watermark) for the
    /// given spawned agent id(s).
    Pending(pending::CommandArgs),
}

impl Commands {
    pub async fn handle(self, cli_config: &crate::Config, handle: &objectiveai_sdk::cli::output::Handle) -> Result<(), crate::error::Error> {
        match self {
            Commands::All(args) => all::handle(args, cli_config, handle).await,
            Commands::Pending(args) => pending::handle(args, cli_config, handle).await,
        }
    }
}
