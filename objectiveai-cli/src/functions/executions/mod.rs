pub mod create;
pub mod logs;
pub mod retry_tokens;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Create a function execution
    Create {
        #[command(subcommand)]
        command: create::Commands,
    },
    /// Print the function-execution instructions and a fresh ID to pass
    /// to `create` via `--instructions-id`.
    Instructions,
    /// Function execution logs
    Logs {
        #[command(subcommand)]
        command: logs::Commands,
    },
    /// Retry tokens
    RetryTokens {
        #[command(subcommand)]
        command: retry_tokens::Commands,
    },
}

impl Commands {
    pub async fn handle(self, cli_config: &crate::Config) -> Result<crate::Output, crate::error::Error> {
        match self {
            Commands::Create { command } => command.handle(cli_config).await,
            Commands::Instructions => Ok(crate::Output::Instructions(
                crate::instructions::format_instructions(include_str!(
                    "../../../assets/functions/executions/create/INSTRUCTIONS.md"
                )),
            )),
            Commands::Logs { command } => command.handle(cli_config).await,
            Commands::RetryTokens { command } => command.handle(cli_config).await,
        }
    }
}
