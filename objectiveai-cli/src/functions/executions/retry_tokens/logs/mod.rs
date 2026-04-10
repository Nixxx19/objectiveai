use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Get a retry token
    Get { filename: String },
}

impl Commands {
    pub async fn handle(self, cli_config: &crate::Config) -> Result<crate::Output, crate::error::Error> {
        let client = objectiveai::filesystem::logs::LogsClient::new(cli_config.config_base_dir.as_deref());
        let content = match self {
            Commands::Get { filename } => client.read_function_execution_retry_token(&filename).await.map(objectiveai::filesystem::logs::LogContent::Json)?,
        };
        Ok(crate::Output::LogsGet(content))
    }
}
