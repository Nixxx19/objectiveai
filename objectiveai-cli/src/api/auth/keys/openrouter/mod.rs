pub mod post;
pub mod get;
pub mod delete;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    Post(post::Args),
    Get,
    Delete,
}

impl Commands {
    pub async fn handle(self, cli_config: &crate::Config, handle: &objectiveai_sdk::cli::output::Handle) -> Result<(), crate::error::Error> {
        match self {
            Commands::Post(args) => post::handle(args, cli_config, handle).await,
            Commands::Get => get::handle(cli_config, handle).await,
            Commands::Delete => delete::handle(cli_config, handle).await,
        }
    }
}
