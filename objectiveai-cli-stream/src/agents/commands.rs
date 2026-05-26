use clap::Subcommand;
use objectiveai_sdk::cli::output::Handle;

use crate::api::{HttpArgs, PipeArgs};

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Agent completions
    Completions {
        #[command(subcommand)]
        command: super::completions::Commands,
    },
}

impl Commands {
    pub async fn handle(
        self,
        http: &HttpArgs,
        pipes: &PipeArgs,
        handle: &Handle,
    ) -> Result<(), String> {
        match self {
            Commands::Completions { command } => command.handle(http, pipes, handle).await,
        }
    }
}
