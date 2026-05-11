use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    Get,
    Set { value: String },
}

impl Commands {
    pub async fn handle(self, cli_config: &crate::Config, handle: &objectiveai_cli_lib::output::Handle) -> Result<(), crate::error::Error> {
        let (client, mut config) = crate::config::read(cli_config).await?;
        match self {
            Commands::Get => {
                crate::config::emit_value(&config.viewer().local().get_signature(), handle).await;
                Ok(())
            },
            Commands::Set { value } => {
                config.viewer().local().set_signature(value);
                crate::config::write(&client, &config, cli_config).await?;
                {
                objectiveai_cli_lib::output::Output::<objectiveai_cli_lib::output::Ok>::Notification(objectiveai_cli_lib::output::Notification { value: objectiveai_cli_lib::output::OK }).emit(handle).await;
                Ok(())
            }
            }
        }
    }
}
