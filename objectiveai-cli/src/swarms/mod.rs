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
    /// List swarms
    List {
        #[command(subcommand)]
        source: crate::list::Source,
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

fn get_favorites() -> Vec<objectiveai::config::Favorite> {
    let (_, mut config) = crate::config::read().unwrap();
    config.swarms().get_favorites().to_vec()
}

async fn list_source(
    http_client: objectiveai::HttpClient,
    source: objectiveai::swarm::request::ListSwarmsSource,
) -> Result<Vec<objectiveai::RemotePath>, crate::error::Error> {
    let response = objectiveai::swarm::list_swarms(
        &http_client,
        objectiveai::swarm::request::ListSwarmsRequest { source: Some(source) },
    ).await?;
    Ok(response.data)
}

impl Commands {
    pub async fn handle(self) -> Result<crate::Output, crate::error::Error> {
        match self {
            Commands::Get { command } => {
                let path: objectiveai::RemotePathCommitOptional = command.into();
                crate::api::run(|http_client| async move {
                    let response = objectiveai::swarm::get_swarm(&http_client, path).await?;
                    Ok(serde_json::to_string_pretty(&response).unwrap())
                }, false).await
            }
            Commands::List { source } => {
                use objectiveai::swarm::request::ListSwarmsSource;
                match source {
                    crate::list::Source::Favorites => crate::list::favorites(get_favorites),
                    crate::list::Source::Filesystem => crate::list::single(|c| Box::pin(list_source(c, ListSwarmsSource::Filesystem))).await,
                    crate::list::Source::Objectiveai => crate::list::single(|c| Box::pin(list_source(c, ListSwarmsSource::Objectiveai))).await,
                    crate::list::Source::All => crate::list::all(
                        get_favorites,
                        |c| Box::pin(list_source(c, ListSwarmsSource::Filesystem)),
                        |c| Box::pin(list_source(c, ListSwarmsSource::Objectiveai)),
                    ).await,
                }
            }
            Commands::Config { command } => command.handle(),
            Commands::Favorites { command } => command.handle(),
        }
    }
}
