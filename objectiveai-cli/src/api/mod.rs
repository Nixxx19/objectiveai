pub mod config;
pub mod mode;
pub mod remote;
pub mod local;
pub mod authorization;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// API configuration
    Config {
        #[command(subcommand)]
        command: config::Commands,
    },
    /// API mode (remote or local)
    Mode {
        #[command(subcommand)]
        command: mode::Commands,
    },
    /// Remote API configuration
    Remote {
        #[command(subcommand)]
        command: remote::Commands,
    },
    /// Local API configuration
    Local {
        #[command(subcommand)]
        command: local::Commands,
    },
    /// Authorization management
    Authorization {
        #[command(subcommand)]
        command: authorization::Commands,
    },
}

impl Commands {
    pub fn handle(self) -> Result<crate::Output, crate::error::Error> {
        match self {
            Commands::Config { command } => command.handle(),
            Commands::Mode { command } => command.handle(),
            Commands::Remote { command } => command.handle(),
            Commands::Local { command } => command.handle(),
            Commands::Authorization { command } => command.handle(),
        }
    }
}
