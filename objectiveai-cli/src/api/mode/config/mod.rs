use clap::{Subcommand, ValueEnum};

#[derive(Clone, ValueEnum)]
pub enum Mode {
    Remote,
    Local,
}

impl From<Mode> for objectiveai::config::ApiMode {
    fn from(m: Mode) -> Self {
        match m {
            Mode::Remote => objectiveai::config::ApiMode::Remote,
            Mode::Local => objectiveai::config::ApiMode::Local,
        }
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Get the mode
    Get,
    /// Set the mode
    Set {
        #[arg(value_enum)]
        value: Mode,
    },
}

impl Commands {
    pub fn handle(self, cli_config: &crate::Config) -> Result<crate::Output, crate::error::Error> {
        let (client, mut config) = crate::config::read()?;
        match self {
            Commands::Get => Ok(crate::Output::ConfigGet(crate::config::format_value(&config.api().get_mode()))),
            Commands::Set { value } => {
                config.api().set_mode(value.into());
                crate::config::write(&client, &config, cli_config)?;
                Ok(crate::Output::ConfigSet)
            }
        }
    }
}
