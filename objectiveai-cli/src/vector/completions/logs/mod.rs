use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Get a vector completion log
    Get { id: String },
    /// Subscribe to changes (wait for create/modify)
    Subscribe {
        id: String,
        #[arg(long)]
        require_modification: bool,
        timeout_ms: u64,
    },
    /// List vector completion logs
    List {
        #[arg(long, default_value_t = 0)]
        offset: usize,
        #[arg(long, default_value_t = 100)]
        limit: usize,
    },
    /// Clear all vector completion logs
    Clear,
}

impl Commands {
    pub async fn handle(self, cli_config: &crate::Config) -> Result<crate::Output, crate::error::Error> {
        let client = objectiveai::filesystem::logs::client::LogsClient::new(cli_config.config_base_dir.as_deref());
        match self {
            Commands::Get { id } => {
                let content = client.read_vector_completion(&id).await.map(objectiveai::filesystem::logs::LogContent::Json)?;
                Ok(crate::Output::LogsGet(content))
            }
            Commands::Subscribe { id, timeout_ms, require_modification } => {
                let result = client.subscribe_vector_completion(&id, std::time::Duration::from_millis(timeout_ms), require_modification).await;
                Ok(crate::Output::LogsSubscribe(result.map(objectiveai::filesystem::logs::LogContent::Json)))
            }
            Commands::List { offset, limit } => Ok(crate::Output::LogsList(client.list_vector_completions(offset, limit).await?)),
            Commands::Clear => Ok(crate::Output::LogsClear(client.clear_vector_completions().await?)),
        }
    }
}
