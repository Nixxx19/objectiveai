mod create;
mod create_args;
pub mod executions;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new laboratory
    Create {
        #[command(flatten)]
        args: create_args::CreateArgs,
    },
    /// Laboratory executions
    Executions {
        #[command(subcommand)]
        command: executions::Commands,
    },
}

impl Commands {
    pub async fn handle(self, cli_config: &crate::Config) -> Result<crate::Output, crate::error::Error> {
        match self {
            Commands::Create { args } => create::handle(args, cli_config).await,
            Commands::Executions { command } => command.handle(cli_config).await,
        }
    }
}
