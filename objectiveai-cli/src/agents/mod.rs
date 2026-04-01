pub mod completions;
pub mod config;
pub mod favorites;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Get an agent by remote path or favorite name
    Get {
        #[command(flatten)]
        args: crate::get::GetArgs,
    },
    /// List agents
    List {
        #[command(subcommand)]
        source: crate::list::Source,
    },
    /// Agent completions
    Completions {
        #[command(subcommand)]
        command: completions::Commands,
    },
    /// Agents configuration
    Config {
        #[command(subcommand)]
        command: config::Commands,
    },
    /// Manage agent favorites
    Favorites {
        #[command(subcommand)]
        command: favorites::Commands,
    },
}

fn get_favorites() -> Vec<objectiveai::config::Favorite> {
    let (_, mut config) = crate::config::read().unwrap();
    config.agents().get_favorites().to_vec()
}

async fn list_source(
    http_client: objectiveai::HttpClient,
    source: objectiveai::agent::request::ListAgentsSource,
) -> Result<Vec<objectiveai::RemotePath>, crate::error::Error> {
    let response = objectiveai::agent::list_agents(
        &http_client,
        objectiveai::agent::request::ListAgentsRequest { source: Some(source) },
    ).await?;
    Ok(response.data)
}

impl Commands {
    pub async fn handle(self) -> Result<crate::Output, crate::error::Error> {
        match self {
            Commands::Get { args } => {
                let path = args.resolve(get_favorites)?;
                crate::api::run(|http_client| async move {
                    let response = objectiveai::agent::get_agent(&http_client, path).await?;
                    Ok(serde_json::to_string(&response).unwrap())
                }, false).await
            }
            Commands::List { source } => {
                use objectiveai::agent::request::ListAgentsSource;
                match source {
                    crate::list::Source::Favorites => crate::list::favorites(get_favorites),
                    crate::list::Source::Filesystem => crate::list::single(|c| Box::pin(list_source(c, ListAgentsSource::Filesystem))).await,
                    crate::list::Source::Objectiveai => crate::list::single(|c| Box::pin(list_source(c, ListAgentsSource::Objectiveai))).await,
                    crate::list::Source::All => crate::list::all(
                        get_favorites,
                        |c| Box::pin(list_source(c, ListAgentsSource::Filesystem)),
                        |c| Box::pin(list_source(c, ListAgentsSource::Objectiveai)),
                    ).await,
                }
            }
            Commands::Completions { command } => command.handle().await,
            Commands::Config { command } => command.handle(),
            Commands::Favorites { command } => command.handle(),
        }
    }
}
