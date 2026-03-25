pub mod config;
pub mod mode;
pub mod remote;
pub mod local;
pub mod headers;
mod run;

pub use run::*;

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
    /// API headers configuration
    Headers {
        #[command(subcommand)]
        command: headers::Commands,
    },
}

impl Commands {
    pub fn handle(self) -> Result<crate::Output, crate::error::Error> {
        match self {
            Commands::Config { command } => command.handle(),
            Commands::Mode { command } => command.handle(),
            Commands::Remote { command } => command.handle(),
            Commands::Local { command } => command.handle(),
            Commands::Headers { command } => command.handle(),
        }
    }
}
