use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Get a message log
    Get { filename: String },
    /// Clear message logs
    Clear {
        /// Also clear nested endpoints (logprobs, image, audio, video, file)
        #[arg(long)]
        nested: bool,
    },
}

impl Commands {
    pub async fn handle(self, cli_config: &crate::Config) -> Result<crate::Output, crate::error::Error> {
        let client = objectiveai::filesystem::logs::LogsClient::new(cli_config.config_base_dir.as_deref());
        match self {
            Commands::Get { filename } => {
                let content = client.read_agent_completion_message(&filename).await.map(objectiveai::filesystem::logs::LogContent::Json)?;
                Ok(crate::Output::LogsGet(content))
            }
            Commands::Clear { nested } => {
                if nested {
                    let counts = futures::future::try_join_all([
                        client.clear_agent_completion_messages(),
                        client.clear_agent_completion_message_logprobs(),
                        client.clear_agent_completion_message_images(),
                        client.clear_agent_completion_message_audio(),
                        client.clear_agent_completion_message_video(),
                        client.clear_agent_completion_message_files(),
                    ]).await?;
                    Ok(crate::Output::LogsClear(counts.into_iter().sum()))
                } else {
                    Ok(crate::Output::LogsClear(client.clear_agent_completion_messages().await?))
                }
            }
        }
    }
}
