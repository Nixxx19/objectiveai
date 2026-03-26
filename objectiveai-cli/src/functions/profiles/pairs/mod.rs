pub mod config;
pub mod favorites;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// List function-profile pairs
    List {
        #[command(subcommand)]
        source: crate::list::Source,
    },
    /// Pairs configuration
    Config {
        #[command(subcommand)]
        command: config::Commands,
    },
    /// Manage pair favorites
    Favorites {
        #[command(subcommand)]
        command: favorites::Commands,
    },
}

fn get_favorites() -> Vec<objectiveai::config::PairFavorite> {
    let (_, mut config) = crate::config::read().unwrap();
    config.functions().profiles().pairs().get_favorites().to_vec()
}

async fn list_objectiveai(
    http_client: objectiveai::HttpClient,
) -> Result<Vec<objectiveai::functions::response::ListFunctionProfilePairItem>, crate::error::Error> {
    let response = objectiveai::functions::list_function_profile_pairs(
        &http_client,
        objectiveai::functions::request::ListFunctionProfilePairsRequest {
            source: Some(objectiveai::functions::request::ListFunctionProfilePairsSource::Objectiveai),
        },
    ).await?;
    Ok(response.data)
}

impl Commands {
    pub async fn handle(self) -> Result<crate::Output, crate::error::Error> {
        match self {
            Commands::List { source } => {
                match source {
                    crate::list::Source::Favorites => crate::list::pair_favorites(get_favorites),
                    crate::list::Source::Filesystem => Err(crate::error::Error::PairsFilesystemNotSupported),
                    crate::list::Source::Objectiveai => crate::list::pair_single(|c| Box::pin(list_objectiveai(c))).await,
                    crate::list::Source::All => crate::list::pair_all(
                        get_favorites,
                        |c| Box::pin(list_objectiveai(c)),
                    ).await,
                }
            }
            Commands::Config { command } => command.handle(),
            Commands::Favorites { command } => command.handle(),
        }
    }
}
