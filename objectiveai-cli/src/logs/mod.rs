use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Get a log file by path
    Get {
        /// Log file path (relative to logs/, e.g. "agent/completions/ac1-abc123.json")
        path: String,
    },
    /// Clear all logs across all endpoints
    Clear,
}

impl Commands {
    pub async fn handle(self, cli_config: &crate::Config) -> Result<crate::Output, crate::error::Error> {
        let client = objectiveai::filesystem::logs::LogsClient::new(cli_config.config_base_dir.as_deref());
        match self {
            Commands::Get { path } => {
                Ok(crate::Output::LogsGet(client.read(&path).await?))
            }
            Commands::Clear => {
                let counts = futures::future::try_join_all([
                    client.clear_agent_completions(),
                    client.clear_agent_completion_continuations(),
                    client.clear_agent_completion_messages(),
                    client.clear_agent_completion_message_logprobs(),
                    client.clear_agent_completion_message_images(),
                    client.clear_agent_completion_message_audio(),
                    client.clear_agent_completion_message_video(),
                    client.clear_agent_completion_message_files(),
                    client.clear_vector_completions(),
                    client.clear_function_executions(),
                    client.clear_function_execution_retry_tokens(),
                    client.clear_function_inventions(),
                    client.clear_function_inventions_recursive(),
                    client.clear_laboratory_executions(),
                ]).await?;
                Ok(crate::Output::LogsClear(counts.into_iter().sum()))
            }
        }
    }
}
