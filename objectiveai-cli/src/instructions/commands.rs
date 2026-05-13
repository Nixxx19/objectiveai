use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Drop every per-scope instructions table, wiping all stored IDs.
    /// Forces every streaming `create` command to require a fresh
    /// `instructions get` round-trip on its next invocation.
    Clear,
}

impl Commands {
    pub async fn handle(self, cli_config: &crate::Config, handle: &objectiveai_cli_sdk::output::Handle) -> Result<(), crate::error::Error> {
        match self {
            Commands::Clear => {
                let count = super::clear_all(cli_config)?;
                {
                #[derive(serde::Serialize)]
                struct Instructions { instructions: String }
                let instructions = format!(
                    "cleared {count} instruction tables"
                );
                objectiveai_cli_sdk::output::Output::<Instructions>::Notification(objectiveai_cli_sdk::output::Notification { value: Instructions { instructions } }).emit(handle).await;
                Ok(())
            }
            }
        }
    }
}
