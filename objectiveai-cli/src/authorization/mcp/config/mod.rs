use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Get all MCP entries
    Get,
    /// Add an MCP authorization entry
    Add { key: String, value: String },
    /// Remove an MCP authorization entry
    Del { key: String },
}

impl Commands {
    pub fn handle(self) -> Result<crate::Output, crate::error::Error> {
        let (client, mut config) = crate::config::read()?;
        match self {
            Commands::Get => Ok(crate::Output::ConfigGet(crate::config::format_value(&config.authorization().get_mcp()))),
            Commands::Add { key, value } => {
                config.authorization().add_mcp(key, value);
                crate::config::write(&client, &config)?;
                Ok(crate::Output::ConfigSet)
            }
            Commands::Del { key } => {
                config.authorization().del_mcp(&key);
                crate::config::write(&client, &config)?;
                Ok(crate::Output::ConfigSet)
            }
        }
    }
}
