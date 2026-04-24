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
    /// Print the laboratory-execution instructions and a fresh ID to pass
    /// to `create` via `--instructions-id`.
    Instructions,
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
            Commands::Instructions => Ok(crate::Output::Instructions(
                crate::instructions::issue(
                    cli_config,
                    crate::instructions::InstructionsScope::LaboratoryExecutions,
                    include_str!("../../../assets/laboratories/executions/create/INSTRUCTIONS.md"),
                )?,
            )),
            Commands::Logs { command } => command.handle(cli_config).await,
        }
    }
}
