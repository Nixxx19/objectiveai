use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Get a message image
    Get { id: String, message_index: u64, media_index: u64 },
    /// Subscribe to changes (wait for create/modify)
    Subscribe { id: String, message_index: u64, media_index: u64, timeout_ms: u64 },
    /// Clear all message images
    Clear,
}

impl Commands {
    pub async fn handle(self, cli_config: &crate::Config) -> Result<crate::Output, crate::error::Error> {
        let client = objectiveai::filesystem::logs::client::LogsClient::new(cli_config.config_base_dir.as_deref());
        match self {
            Commands::Get { id, message_index, media_index } => {
                let content = client.read_agent_completion_message_image(&id, message_index, media_index).await.map(objectiveai::filesystem::logs::LogContent::DataUrl)?;
                Ok(crate::Output::LogsGet(content))
            }
            Commands::Subscribe { id, message_index, media_index, timeout_ms } => {
                let result = client.subscribe_agent_completion_message_image(&id, message_index, media_index, std::time::Duration::from_millis(timeout_ms)).await;
                Ok(crate::Output::LogsSubscribe(result.map(objectiveai::filesystem::logs::LogContent::DataUrl)))
            }
            Commands::Clear => Ok(crate::Output::LogsClear(client.clear_agent_completion_message_images().await?)),
        }
    }
}
