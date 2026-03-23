use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Get the value
    Get,
    /// Set the value
    Set { value: String },
}

impl Commands {
    pub fn handle(self) -> Result<Option<String>, crate::error::Error> {
        let (client, mut config) = crate::config::read()?;
        match self {
            Commands::Get => Ok(Some(crate::config::format_value(&config.authorization().get_github()))),
            Commands::Set { value } => {
                config.authorization().set_github(value);
                crate::config::write(&client, &config)?;
                Ok(None)
            }
        }
    }
}
