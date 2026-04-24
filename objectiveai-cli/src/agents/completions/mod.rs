pub mod create;
pub mod logs;
pub mod continuations;
pub mod messages;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Create an agent completion
    Create {
        #[command(subcommand)]
        command: create::Commands,
    },
    /// Print the agent-completion instructions and a fresh ID to pass
    /// to `create` via `--instructions-id`.
    Instructions,
    /// Agent completion logs
    Logs {
        #[command(subcommand)]
        command: logs::Commands,
    },
    /// Agent completion continuations
    Continuations {
        #[command(subcommand)]
        command: continuations::Commands,
    },
    /// Agent completion messages
    Messages {
        #[command(subcommand)]
        command: messages::Commands,
    },
}

impl Commands {
    pub async fn handle(self, cli_config: &crate::Config) -> Result<crate::Output, crate::error::Error> {
        match self {
            Commands::Create { command } => command.handle(cli_config).await,
            Commands::Instructions => Ok(crate::Output::Instructions(
                crate::instructions::issue(
                    cli_config,
                    crate::instructions::InstructionsScope::AgentCompletions,
                    include_str!("../../../assets/agents/completions/create/INSTRUCTIONS.md"),
                )?,
            )),
            Commands::Logs { command } => command.handle(cli_config).await,
            Commands::Continuations { command } => command.handle(cli_config).await,
            Commands::Messages { command } => command.handle(cli_config).await,
        }
    }
}
