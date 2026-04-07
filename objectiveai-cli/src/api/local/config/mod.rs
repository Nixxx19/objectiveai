use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Query local API config using jq syntax
    Get { filter: Option<String> },
}

impl Commands {
    pub fn handle(self) -> Result<crate::Output, crate::error::Error> {
        let (_, mut config) = crate::config::read()?;
        match self {
            Commands::Get { filter } => crate::config::format_jq(config.api().local().jq(&crate::config::filter(filter))),
        }
    }
}
