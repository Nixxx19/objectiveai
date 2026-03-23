use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Query authorization config using jq syntax
    Get { filter: Option<String> },
}

impl Commands {
    pub fn handle(self) -> Result<Option<String>, crate::error::Error> {
        let (_, mut config) = crate::config::read()?;
        match self {
            Commands::Get { filter } => crate::config::format_jq(config.authorization().jq(&crate::config::filter(filter))),
        }
    }
}
