use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Get the value
    Get,
    /// Set the value
    Set { value: String },
}

impl Commands {
    pub fn handle(self) -> Result<crate::Output, crate::error::Error> {
        let (client, mut config) = crate::config::read()?;
        match self {
            Commands::Get => Ok(crate::Output::ConfigGet(crate::config::format_value(&config.authorization().get_objectiveai()))),
            Commands::Set { value } => {
                config.authorization().set_objectiveai(value);
                crate::config::write(&client, &config)?;
                Ok(crate::Output::ConfigSet)
            }
        }
    }
}
