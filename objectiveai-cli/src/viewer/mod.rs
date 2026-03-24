pub mod config;
pub mod mode;
pub mod local;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Viewer configuration
    Config {
        #[command(subcommand)]
        command: config::Commands,
    },
    /// Viewer mode (disabled or local)
    Mode {
        #[command(subcommand)]
        command: mode::Commands,
    },
    /// Local viewer configuration
    Local {
        #[command(subcommand)]
        command: local::Commands,
    },
}

impl Commands {
    pub fn handle(self) -> Result<crate::Output, crate::error::Error> {
        match self {
            Commands::Config { command } => command.handle(),
            Commands::Mode { command } => command.handle(),
            Commands::Local { command } => command.handle(),
        }
    }
}
