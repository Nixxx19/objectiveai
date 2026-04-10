use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    Get,
    Set { value: String },
}

impl Commands {
    pub async fn handle(self, cli_config: &crate::Config) -> Result<crate::Output, crate::error::Error> {
        let (client, mut config) = crate::config::read(cli_config).await?;
        match self {
            Commands::Get => Ok(crate::Output::ConfigGet(crate::config::format_value(&config.api().headers().get_http_referer()))),
            Commands::Set { value } => {
                config.api().headers().set_http_referer(value);
                crate::config::write(&client, &config, cli_config).await?;
                Ok(crate::Output::ConfigSet)
            }
        }
    }
}
