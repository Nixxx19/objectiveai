use clap::{Subcommand, ValueEnum};

#[derive(Clone, ValueEnum)]
pub enum Mode {
    Remote,
    Local,
}

impl From<Mode> for objectiveai::filesystem::config::ViewerMode {
    fn from(m: Mode) -> Self {
        match m {
            Mode::Remote => objectiveai::filesystem::config::ViewerMode::Remote,
            Mode::Local => objectiveai::filesystem::config::ViewerMode::Local,
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
    pub async fn handle(self, cli_config: &crate::Config, handle: &objectiveai_cli_lib::output::Handle) -> Result<(), crate::error::Error> {
        let (client, mut config) = crate::config::read(cli_config).await?;
        match self {
            Commands::Get => {
                crate::config::emit_value(&config.viewer().get_mode(), handle).await;
                Ok(())
            },
            Commands::Set { value } => {
                config.viewer().set_mode(value.into());
                crate::config::write(&client, &config, cli_config).await?;
                {
                objectiveai_cli_lib::output::Output::<objectiveai_cli_lib::output::Ok>::Notification(objectiveai_cli_lib::output::Notification { value: objectiveai_cli_lib::output::OK }).emit(handle).await;
                Ok(())
            }
            }
        }
    }
}
