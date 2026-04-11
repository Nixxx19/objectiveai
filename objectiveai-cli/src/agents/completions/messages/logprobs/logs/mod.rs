use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Get message logprobs
    Get { id: String, message_index: u64 },
    /// Clear all message logprobs
    Clear,
}

impl Commands {
    pub async fn handle(self, cli_config: &crate::Config) -> Result<crate::Output, crate::error::Error> {
        let client = objectiveai::filesystem::logs::LogsClient::new(cli_config.config_base_dir.as_deref());
        match self {
            Commands::Get { id, message_index } => {
                let content = client.read_agent_completion_message_logprobs(&id, message_index).await.map(objectiveai::filesystem::logs::LogContent::Json)?;
                Ok(crate::Output::LogsGet(content))
            }
            Commands::Clear => Ok(crate::Output::LogsClear(client.clear_agent_completion_message_logprobs().await?)),
        }
    }
}
