use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Get the value
    Get,
    /// Set the value
    Set { value: String },
}

impl Commands {
    pub fn handle(self, cli_config: &crate::Config) -> Result<crate::Output, crate::error::Error> {
        let (client, mut config) = crate::config::read()?;
        match self {
            Commands::Get => Ok(crate::Output::ConfigGet(crate::config::format_value(&config.api().remote().get_objectiveai_address()))),
            Commands::Set { value } => {
                config.api().remote().set_objectiveai_address(value);
                crate::config::write(&client, &config, cli_config)?;
                Ok(crate::Output::ConfigSet)
            }
        }
    }
}
