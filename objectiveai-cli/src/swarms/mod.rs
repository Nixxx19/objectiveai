pub mod config;
pub mod favorites;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Get a swarm by remote path
    Get {
        #[command(subcommand)]
        command: crate::path::RemotePathCommitOptional,
    },
    /// Swarms configuration
    Config {
        #[command(subcommand)]
        command: config::Commands,
    },
    /// Manage swarm favorites
    Favorites {
        #[command(subcommand)]
        command: favorites::Commands,
    },
}

impl Commands {
    pub async fn handle(self) -> Result<crate::Output, crate::error::Error> {
        match self {
            Commands::Get { command } => {
                let path: objectiveai::RemotePathCommitOptional = command.into();
                crate::api::run(|http_client| async move {
                    let response = objectiveai::swarm::get_swarm(&http_client, path).await?;
                    Ok(serde_json::to_string_pretty(&response).unwrap())
                }).await
            }
            Commands::Config { command } => command.handle(),
            Commands::Favorites { command } => command.handle(),
        }
    }
}
