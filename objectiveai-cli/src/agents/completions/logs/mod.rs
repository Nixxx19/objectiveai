use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Get an agent completion log
    Get { id: String },
    /// Subscribe to changes (wait for create/modify)
    Subscribe {
        id: String,
        #[arg(long)]
        require_modification: bool,
        timeout_ms: u64,
    },
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
        let client = objectiveai::filesystem::logs::client::LogsClient::new(cli_config.config_base_dir.as_deref());
        match self {
            Commands::Get { id } => {
                let content = client.read_agent_completion(&id).await.map(objectiveai::filesystem::logs::LogContent::Json)?;
                Ok(crate::Output::LogsGet(content))
            }
            Commands::Subscribe { id, timeout_ms, require_modification } => {
                let result = client.subscribe_agent_completion(&id, std::time::Duration::from_millis(timeout_ms), require_modification).await;
                Ok(crate::Output::LogsSubscribe(result.map(objectiveai::filesystem::logs::LogContent::Json)))
            }
            Commands::List { offset, limit } => {
                Ok(crate::Output::LogsList(client.list_agent_completions(offset, limit).await?))
            }
            Commands::Clear { nested } => {
                if nested {
                    let counts = futures::future::try_join_all(vec![
                        Box::pin(client.clear_agent_completions()) as std::pin::Pin<Box<dyn std::future::Future<Output = _>>>,
                        Box::pin(client.clear_agent_completion_continuations()),
                        Box::pin(client.clear_agent_completion_messages()),
                        Box::pin(client.clear_agent_completion_message_logprobs()),
                        Box::pin(client.clear_agent_completion_message_images()),
                        Box::pin(client.clear_agent_completion_message_audio()),
                        Box::pin(client.clear_agent_completion_message_video()),
                        Box::pin(client.clear_agent_completion_message_files()),
                    ]).await?;
                    Ok(crate::Output::LogsClear(counts.into_iter().sum()))
                } else {
                    Ok(crate::Output::LogsClear(client.clear_agent_completions().await?))
                }
            }
        }
    }
}
