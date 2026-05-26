use clap::Subcommand;
use objectiveai_sdk::cli::output::Handle;

use crate::api::{HttpArgs, PipeArgs};

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Laboratory executions
    Executions {
        #[command(subcommand)]
        command: super::executions::Commands,
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
            Commands::Executions { command } => command.handle(http, pipes, handle).await,
        }
    }
}
