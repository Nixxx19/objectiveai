pub mod config;
pub mod favorites;
pub mod pairs;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Get a profile by remote path
    Get {
        #[command(subcommand)]
        command: crate::path::RemotePathCommitOptional,
    },
    /// List profiles
    List {
        #[command(subcommand)]
        source: crate::list::Source,
    },
    /// Profiles configuration
    Config {
        #[command(subcommand)]
        command: config::Commands,
    },
    /// Manage profile favorites
    Favorites {
        #[command(subcommand)]
        command: favorites::Commands,
    },
    /// Function-profile pairs
    Pairs {
        #[command(subcommand)]
        command: pairs::Commands,
    },
}

fn get_favorites() -> Vec<objectiveai::config::Favorite> {
    let (_, mut config) = crate::config::read().unwrap();
    config.functions().profiles().get_favorites().to_vec()
}

async fn list_source(
    http_client: objectiveai::HttpClient,
    source: objectiveai::functions::profiles::request::ListProfilesSource,
) -> Result<Vec<objectiveai::RemotePath>, crate::error::Error> {
    let response = objectiveai::functions::profiles::list_profiles(
        &http_client,
        objectiveai::functions::profiles::request::ListProfilesRequest { source: Some(source) },
    ).await?;
    Ok(response.data)
}

impl Commands {
    pub async fn handle(self) -> Result<crate::Output, crate::error::Error> {
        match self {
            Commands::Get { command } => {
                let path: objectiveai::RemotePathCommitOptional = command.into();
                crate::api::run(|http_client| async move {
                    let response = objectiveai::functions::profiles::get_profile(&http_client, path).await?;
                    Ok(serde_json::to_string_pretty(&response).unwrap())
                }, false).await
            }
            Commands::List { source } => {
                use objectiveai::functions::profiles::request::ListProfilesSource;
                match source {
                    crate::list::Source::Favorites => crate::list::favorites(get_favorites),
                    crate::list::Source::Filesystem => crate::list::single(|c| Box::pin(list_source(c, ListProfilesSource::Filesystem))).await,
                    crate::list::Source::Objectiveai => crate::list::single(|c| Box::pin(list_source(c, ListProfilesSource::Objectiveai))).await,
                    crate::list::Source::All => crate::list::all(
                        get_favorites,
                        |c| Box::pin(list_source(c, ListProfilesSource::Filesystem)),
                        |c| Box::pin(list_source(c, ListProfilesSource::Objectiveai)),
                    ).await,
                }
            }
            Commands::Config { command } => command.handle(),
            Commands::Favorites { command } => command.handle(),
            Commands::Pairs { command } => command.handle(),
        }
    }
}
