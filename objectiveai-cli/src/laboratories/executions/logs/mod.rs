use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Get a laboratory execution log
    Get { id: String },
    /// List laboratory execution logs
    List {
        #[arg(long, default_value_t = 0)]
        offset: usize,
        #[arg(long, default_value_t = 100)]
        limit: usize,
    },
    /// Clear all laboratory execution logs
    Clear,
}

impl Commands {
    pub async fn handle(self, cli_config: &crate::Config) -> Result<crate::Output, crate::error::Error> {
        let client = objectiveai::filesystem::logs::LogsClient::new(cli_config.config_base_dir.as_deref());
        match self {
            Commands::Get { id } => {
                let content = client.read_laboratory_execution(&id).await.map(objectiveai::filesystem::logs::LogContent::Json)?;
                Ok(crate::Output::LogsGet(content))
            }
            Commands::List { offset, limit } => {
                Ok(crate::Output::LogsList(client.list_laboratory_executions(offset, limit).await?))
            }
            Commands::Clear => Ok(crate::Output::LogsClear(client.clear_laboratory_executions().await?)),
        }
    }
}
