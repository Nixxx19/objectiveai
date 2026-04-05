use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    Get,
    Set { value: String },
}

impl Commands {
    pub fn handle(self, cli_config: &crate::Config) -> Result<crate::Output, crate::error::Error> {
        let (client, mut config) = crate::config::read()?;
        match self {
            Commands::Get => Ok(crate::Output::ConfigGet(crate::config::format_value(&config.api().headers().get_x_commit_author_name()))),
            Commands::Set { value } => {
                config.api().headers().set_x_commit_author_name(value);
                crate::config::write(&client, &config, cli_config)?;
                Ok(crate::Output::ConfigSet)
            }
        }
    }
}
