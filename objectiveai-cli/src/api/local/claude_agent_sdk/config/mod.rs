use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Get the value
    Get,
    /// Set the value
    Set { value: bool },
}

impl Commands {
    pub async fn handle(self, cli_config: &crate::Config) -> Result<crate::Output, crate::error::Error> {
        let (client, mut config) = crate::config::read(cli_config).await?;
        match self {
            Commands::Get => Ok(crate::Output::ConfigGet(crate::config::format_value(&config.api().local().get_claude_agent_sdk()))),
            Commands::Set { value } => {
                config.api().local().set_claude_agent_sdk(value);
                crate::config::write(&client, &config, cli_config).await?;
                Ok(crate::Output::ConfigSet)
            }
        }
    }
}
