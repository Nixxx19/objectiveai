use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Get an agent completion log
    Get { filename: String },
    /// List agent completion logs
    List {
        #[arg(long, default_value_t = 0)]
        offset: usize,
        #[arg(long, default_value_t = 100)]
        limit: usize,
    },
    /// Clear agent completion logs
    Clear {
        /// Also clear nested endpoints (continuations, messages, etc.)
        #[arg(long)]
        nested: bool,
    },
}

impl Commands {
    pub async fn handle(self, cli_config: &crate::Config) -> Result<crate::Output, crate::error::Error> {
        let client = objectiveai::filesystem::logs::LogsClient::new(cli_config.config_base_dir.as_deref());
        match self {
            Commands::Get { filename } => {
                let content = client.read_agent_completion(&filename).await.map(objectiveai::filesystem::logs::LogContent::Json)?;
                Ok(crate::Output::LogsGet(content))
            }
            Commands::List { offset, limit } => {
                Ok(crate::Output::LogsList(client.list_agent_completions(offset, limit).await?))
            }
            Commands::Clear { nested } => {
                if nested {
                    let counts = futures::future::try_join_all([
                        client.clear_agent_completions(),
                        client.clear_agent_completion_continuations(),
                        client.clear_agent_completion_messages(),
                        client.clear_agent_completion_message_logprobs(),
                        client.clear_agent_completion_message_images(),
                        client.clear_agent_completion_message_audio(),
                        client.clear_agent_completion_message_video(),
                        client.clear_agent_completion_message_files(),
                    ]).await?;
                    Ok(crate::Output::LogsClear(counts.into_iter().sum()))
                } else {
                    Ok(crate::Output::LogsClear(client.clear_agent_completions().await?))
                }
            }
        }
    }
}
