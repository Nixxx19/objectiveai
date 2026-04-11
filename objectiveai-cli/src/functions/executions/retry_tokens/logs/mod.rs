use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Get a retry token
    Get { id: String },
    /// Subscribe to changes (wait for create/modify)
    Subscribe { id: String, timeout_ms: u64 },
    /// Clear all retry tokens
    Clear,
}

impl Commands {
    pub async fn handle(self, cli_config: &crate::Config) -> Result<crate::Output, crate::error::Error> {
        let client = objectiveai::filesystem::logs::LogsClient::new(cli_config.config_base_dir.as_deref());
        match self {
            Commands::Get { id } => {
                let content = client.read_function_execution_retry_token(&id).await.map(objectiveai::filesystem::logs::LogContent::Json)?;
                Ok(crate::Output::LogsGet(content))
            }
            Commands::Subscribe { id, timeout_ms } => {
                let result = client.subscribe_function_execution_retry_token(&id, std::time::Duration::from_millis(timeout_ms)).await;
                Ok(crate::Output::LogsSubscribe(result.map(objectiveai::filesystem::logs::LogContent::Json)))
            }
            Commands::Clear => Ok(crate::Output::LogsClear(client.clear_function_execution_retry_tokens().await?)),
        }
    }
}
