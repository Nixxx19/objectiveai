use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Get the secret
    Get,
    /// Set the secret
    Set { value: String },
}

impl Commands {
    pub fn handle(self) -> Result<crate::Output, crate::error::Error> {
        let (client, mut config) = crate::config::read()?;
        match self {
            Commands::Get => Ok(crate::Output::ConfigGet(crate::config::format_value(&config.viewer().get_secret()))),
            Commands::Set { value } => {
                config.viewer().set_secret(value);
                crate::config::write(&client, &config)?;
                Ok(crate::Output::ConfigSet)
            }
        }
    }
}
