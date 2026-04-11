use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Get a message video
    Get { id: String, message_index: u64, media_index: u64 },
    /// Clear all message video
    Clear,
}

impl Commands {
    pub async fn handle(self, cli_config: &crate::Config) -> Result<crate::Output, crate::error::Error> {
        let client = objectiveai::filesystem::logs::LogsClient::new(cli_config.config_base_dir.as_deref());
        match self {
            Commands::Get { id, message_index, media_index } => {
                let content = client.read_agent_completion_message_video(&id, message_index, media_index).await.map(objectiveai::filesystem::logs::LogContent::DataUrl)?;
                Ok(crate::Output::LogsGet(content))
            }
            Commands::Clear => Ok(crate::Output::LogsClear(client.clear_agent_completion_message_video().await?)),
        }
    }
}
