pub mod create;
pub mod logs;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Create a recursive function invention
    Create {
        #[command(subcommand)]
        command: create::Commands,
    },
    /// Print the recursive-invention instructions and a fresh ID to pass
    /// to `create` via `--instructions-id`.
    Instructions,
    /// Read recursive invention logs
    Logs {
        #[command(subcommand)]
        command: logs::Commands,
    },
}

impl Commands {
    pub async fn handle(self, cli_config: &crate::Config) -> Result<crate::Output, crate::error::Error> {
        match self {
            Commands::Create { command } => command.handle(cli_config).await,
            Commands::Instructions => Ok(crate::Output::Instructions(
                crate::instructions::issue(
                    cli_config,
                    crate::instructions::InstructionsScope::FunctionInventionsRecursive,
                    include_str!("../../../../assets/functions/inventions/recursive/create/INSTRUCTIONS.md"),
                )?,
            )),
            Commands::Logs { command } => command.handle(cli_config).await,
        }
    }
}
