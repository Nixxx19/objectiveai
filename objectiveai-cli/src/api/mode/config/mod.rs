use clap::{Subcommand, ValueEnum};

#[derive(Clone, ValueEnum)]
pub enum Mode {
    Remote,
    Local,
}

impl From<Mode> for objectiveai::ApiMode {
    fn from(m: Mode) -> Self {
        match m {
            Mode::Remote => objectiveai::ApiMode::Remote,
            Mode::Local => objectiveai::ApiMode::Local,
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
    pub fn handle(self) -> Result<Option<String>, crate::error::Error> {
        let (client, mut config) = crate::config::read()?;
        match self {
            Commands::Get => Ok(Some(crate::config::format_value(&config.api().get_mode()))),
            Commands::Set { value } => {
                config.api().set_mode(value.into());
                crate::config::write(&client, &config)?;
                Ok(None)
            }
        }
    }
}
