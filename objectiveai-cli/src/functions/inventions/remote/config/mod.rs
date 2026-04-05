use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Get the remote
    Get,
    /// Set the remote
    Set {
        #[arg(value_enum)]
        value: super::Remote,
    },
}

impl Commands {
    pub fn handle(self, cli_config: &crate::Config) -> Result<crate::Output, crate::error::Error> {
        let (client, mut config) = crate::config::read()?;
        match self {
            Commands::Get => Ok(crate::Output::ConfigGet(crate::config::format_value(&config.functions().inventions().get_remote()))),
            Commands::Set { value } => {
                config.functions().inventions().set_remote(value.into())?;
                crate::config::write(&client, &config, cli_config)?;
                Ok(crate::Output::ConfigSet)
            }
        }
    }
}
