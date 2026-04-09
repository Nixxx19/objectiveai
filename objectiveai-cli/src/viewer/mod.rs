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
    /// Viewer mode (remote or local)
    Mode {
        #[command(subcommand)]
        command: mode::Commands,
    },
    /// Local viewer configuration
    Local {
        #[command(subcommand)]
        command: local::Commands,
    },
    /// Generate a new secret/signature pair
    GenerateSecretSignaturePair,
}

impl Commands {
    pub async fn handle(self, cli_config: &crate::Config) -> Result<crate::Output, crate::error::Error> {
        match self {
            Commands::Config { command } => command.handle(cli_config).await,
            Commands::Mode { command } => command.handle(cli_config).await,
            Commands::Local { command } => command.handle(cli_config).await,
            Commands::GenerateSecretSignaturePair => {
                let pair = objectiveai::filesystem::config::generate_viewer_secret_signature_pair();
                Ok(crate::Output::ConfigGet(serde_json::to_string(&pair).unwrap()))
            }
        }
    }
}
