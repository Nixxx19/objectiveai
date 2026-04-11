use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Get a function execution log
    Get { id: String },
    /// Subscribe to changes (wait for create/modify)
    Subscribe { id: String, timeout_ms: u64 },
    /// List function execution logs
    List {
        #[arg(long, default_value_t = 0)]
        offset: usize,
        #[arg(long, default_value_t = 100)]
        limit: usize,
    },
    /// Clear function execution logs
    Clear {
        /// Also clear nested endpoints (retry tokens)
        #[arg(long)]
        nested: bool,
    },
}

impl Commands {
    pub async fn handle(self, cli_config: &crate::Config) -> Result<crate::Output, crate::error::Error> {
        let client = objectiveai::filesystem::logs::client::LogsClient::new(cli_config.config_base_dir.as_deref());
        match self {
            Commands::Get { id } => {
                let content = client.read_function_execution(&id).await.map(objectiveai::filesystem::logs::LogContent::Json)?;
                Ok(crate::Output::LogsGet(content))
            }
            Commands::Subscribe { id, timeout_ms } => {
                let result = client.subscribe_function_execution(&id, std::time::Duration::from_millis(timeout_ms)).await;
                Ok(crate::Output::LogsSubscribe(result.map(objectiveai::filesystem::logs::LogContent::Json)))
            }
            Commands::List { offset, limit } => Ok(crate::Output::LogsList(client.list_function_executions(offset, limit).await?)),
            Commands::Clear { nested } => {
                if nested {
                    let counts = futures::future::try_join_all(vec![
                        Box::pin(client.clear_function_executions()) as std::pin::Pin<Box<dyn std::future::Future<Output = _>>>,
                        Box::pin(client.clear_function_execution_retry_tokens()),
                    ]).await?;
                    Ok(crate::Output::LogsClear(counts.into_iter().sum()))
                } else {
                    Ok(crate::Output::LogsClear(client.clear_function_executions().await?))
                }
            }
        }
    }
}
