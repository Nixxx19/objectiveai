use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Get a function invention log
    Get { id: String },
    /// Subscribe to changes (wait for create/modify)
    Subscribe { id: String, timeout_ms: u64 },
    /// List function invention logs
    List {
        #[arg(long, default_value_t = 0)]
        offset: usize,
        #[arg(long, default_value_t = 100)]
        limit: usize,
    },
    /// Clear all function invention logs
    Clear,
}

impl Commands {
    pub async fn handle(self, cli_config: &crate::Config) -> Result<crate::Output, crate::error::Error> {
        let client = objectiveai::filesystem::logs::client::LogsClient::new(cli_config.config_base_dir.as_deref());
        match self {
            Commands::Get { id } => {
                let content = client.read_function_invention(&id).await.map(objectiveai::filesystem::logs::LogContent::Json)?;
                Ok(crate::Output::LogsGet(content))
            }
            Commands::Subscribe { id, timeout_ms } => {
                let result = client.subscribe_function_invention(&id, std::time::Duration::from_millis(timeout_ms)).await;
                Ok(crate::Output::LogsSubscribe(result.map(objectiveai::filesystem::logs::LogContent::Json)))
            }
            Commands::List { offset, limit } => Ok(crate::Output::LogsList(client.list_function_inventions(offset, limit).await?)),
            Commands::Clear => Ok(crate::Output::LogsClear(client.clear_function_inventions().await?)),
        }
    }
}
