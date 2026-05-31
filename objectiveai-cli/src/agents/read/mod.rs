pub mod all;
pub mod id;
pub mod pending;
pub mod subscribe;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Return every queue item for the given spawned agent id(s).
    /// Still advances the pending watermark to the highest index,
    /// so a subsequent `read pending` returns empty until new rows
    /// land.
    All(all::CommandArgs),
    /// Return the file content for a queue Id (SQL row id from the
    /// `files` table).
    Id(id::CommandArgs),
    /// Drain pending queue items (past the watermark) for the
    /// given spawned agent id(s).
    Pending(pending::CommandArgs),
    /// Block on the spawned agent's next queue event. Emits any
    /// pending unread items immediately, then waits (no polling)
    /// for new rows or stream end. With `--kind`, only returns on
    /// a matching item or stream end. Emits `Inactive` when no
    /// active stream exists and nothing is unread.
    Subscribe(subscribe::CommandArgs),
}

impl Commands {
    pub async fn handle(
        self,
        cli_config: &crate::Config,
        handle: &objectiveai_sdk::cli::output::Handle,
    ) -> Result<(), crate::error::Error> {
        match self {
            Commands::All(args) => all::handle(args, cli_config, handle).await,
            Commands::Id(args) => id::handle(args, cli_config, handle).await,
            Commands::Pending(args) => pending::handle(args, cli_config, handle).await,
            Commands::Subscribe(args) => subscribe::handle(args, cli_config, handle).await,
        }
    }
}
