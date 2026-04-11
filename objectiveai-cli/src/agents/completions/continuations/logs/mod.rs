use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Get a continuation log
    Get { id: String },
    /// Subscribe to changes (wait for create/modify)
    Subscribe { id: String, timeout_ms: u64 },
    /// Clear all continuation logs
    Clear,
}

impl Commands {
    pub async fn handle(self, cli_config: &crate::Config) -> Result<crate::Output, crate::error::Error> {
        let client = objectiveai::filesystem::logs::LogsClient::new(cli_config.config_base_dir.as_deref());
        match self {
            Commands::Get { id } => {
                let content = client.read_agent_completion_continuation(&id).await.map(objectiveai::filesystem::logs::LogContent::Json)?;
                Ok(crate::Output::LogsGet(content))
            }
            Commands::Subscribe { id, timeout_ms } => {
                let result = client.subscribe_agent_completion_continuation(&id, std::time::Duration::from_millis(timeout_ms)).await;
                Ok(crate::Output::LogsSubscribe(result.map(objectiveai::filesystem::logs::LogContent::Json)))
            }
            Commands::Clear => Ok(crate::Output::LogsClear(client.clear_agent_completion_continuations().await?)),
        }
    }
}
