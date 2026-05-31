pub mod active;
pub mod available;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// List every direct-child agent of a parent agent id along with
    /// the timestamp of its most recent `assistant_response` row.
    /// Direct children only; deeper descendants (composite ids with
    /// more than one segment past the parent) are excluded.
    Active(active::CommandArgs),
    /// List remote agents from one or more sources (favorites,
    /// filesystem, ObjectiveAI, mock, or the unioned view).
    Available {
        #[command(subcommand)]
        source: crate::list::Source,
    },
}

impl Commands {
    pub async fn handle(
        self,
        cli_config: &crate::Config,
        handle: &objectiveai_sdk::cli::output::Handle,
    ) -> Result<(), crate::error::Error> {
        match self {
            Commands::Active(args) => active::handle(args, cli_config, handle).await,
            Commands::Available { source } => available::handle(source, cli_config, handle).await,
        }
    }
}
