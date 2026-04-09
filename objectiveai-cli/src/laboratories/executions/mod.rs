mod create;
mod create_args;
pub mod logs;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Create a laboratory execution
    Create {
        #[command(flatten)]
        args: create_args::CreateArgs,
    },
    /// Laboratory execution logs
    Logs {
        #[command(subcommand)]
        command: logs::Commands,
    },
}

impl Commands {
    pub async fn handle(self, cli_config: &crate::Config) -> Result<crate::Output, crate::error::Error> {
        match self {
            Commands::Create { args } => create::handle(args, cli_config).await,
            Commands::Logs { command } => command.handle(cli_config).await,
        }
    }
}
