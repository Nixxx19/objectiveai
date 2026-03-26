pub mod config;
pub mod favorites;
pub mod inventions;
pub mod profiles;
pub mod executions;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Get a function by remote path or favorite name
    Get {
        #[command(flatten)]
        args: crate::get::GetArgs,
    },
    /// List functions
    List {
        #[command(subcommand)]
        source: crate::list::Source,
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

fn get_favorites() -> Vec<objectiveai::config::Favorite> {
    let (_, mut config) = crate::config::read().unwrap();
    config.functions().get_favorites().to_vec()
}

async fn list_source(
    http_client: objectiveai::HttpClient,
    source: objectiveai::functions::request::ListFunctionsSource,
) -> Result<Vec<objectiveai::RemotePath>, crate::error::Error> {
    let response = objectiveai::functions::list_functions(
        &http_client,
        objectiveai::functions::request::ListFunctionsRequest { source: Some(source) },
    ).await?;
    Ok(response.data)
}

impl Commands {
    pub async fn handle(self) -> Result<crate::Output, crate::error::Error> {
        match self {
            Commands::Get { args } => {
                let path = args.resolve(get_favorites)?;
                crate::api::run(|http_client| async move {
                    let response = objectiveai::functions::get_function(&http_client, path).await?;
                    Ok(serde_json::to_string_pretty(&response).unwrap())
                }, false).await
            }
            Commands::List { source } => {
                use objectiveai::functions::request::ListFunctionsSource;
                match source {
                    crate::list::Source::Favorites => crate::list::favorites(get_favorites),
                    crate::list::Source::Filesystem => crate::list::single(|c| Box::pin(list_source(c, ListFunctionsSource::Filesystem))).await,
                    crate::list::Source::Objectiveai => crate::list::single(|c| Box::pin(list_source(c, ListFunctionsSource::Objectiveai))).await,
                    crate::list::Source::All => crate::list::all(
                        get_favorites,
                        |c| Box::pin(list_source(c, ListFunctionsSource::Filesystem)),
                        |c| Box::pin(list_source(c, ListFunctionsSource::Objectiveai)),
                    ).await,
                }
            }
            Commands::Config { command } => command.handle(),
            Commands::Favorites { command } => command.handle(),
            Commands::Inventions { command } => command.handle(),
            Commands::Profiles { command } => command.handle().await,
        }
    }
}
