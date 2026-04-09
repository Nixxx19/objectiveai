use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Query local viewer config using jq syntax
    Get { filter: Option<String> },
}

impl Commands {
    pub async fn handle(self, cli_config: &crate::Config) -> Result<crate::Output, crate::error::Error> {
        let (_, mut config) = crate::config::read(cli_config).await?;
        match self {
            Commands::Get { filter } => crate::config::format_jq(config.viewer().local().jq(&crate::config::filter(filter))),
        }
    }
}
