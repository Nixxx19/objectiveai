pub mod config;
pub mod favorites;
pub mod inventions;
pub mod profiles;
pub mod executions;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Get a function by remote path
    Get {
        #[command(subcommand)]
        command: crate::path::RemotePathCommitOptional,
    },
    /// Functions configuration
    Config {
        #[command(subcommand)]
        command: config::Commands,
    },
    /// Manage function favorites
    Favorites {
        #[command(subcommand)]
        command: favorites::Commands,
    },
    /// Functions inventions
    Inventions {
        #[command(subcommand)]
        command: inventions::Commands,
    },
    /// Functions profiles
    Profiles {
        #[command(subcommand)]
        command: profiles::Commands,
    },
}

impl Commands {
    pub async fn handle(self) -> Result<crate::Output, crate::error::Error> {
        match self {
            Commands::Get { command } => {
                let path: objectiveai::RemotePathCommitOptional = command.into();
                crate::api::run(|http_client| async move {
                    let response = objectiveai::functions::get_function(&http_client, path).await?;
                    Ok(serde_json::to_string_pretty(&response).unwrap())
                }).await
            }
            Commands::Config { command } => command.handle(),
            Commands::Favorites { command } => command.handle(),
            Commands::Inventions { command } => command.handle(),
            Commands::Profiles { command } => command.handle().await,
        }
    }
}
