use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Get a log file by path
    Get {
        /// Log file path (relative to logs/, e.g. "agent/completions/ac1-abc123.json")
        path: String,
    },
}

impl Commands {
    pub async fn handle(self, cli_config: &crate::Config) -> Result<crate::Output, crate::error::Error> {
        let client = objectiveai::filesystem::logs::LogsClient::new(cli_config.config_base_dir.as_deref());
        let content = match self {
            Commands::Get { path } => client.read(&path).await?,
        };
        Ok(crate::Output::LogsGet(content))
    }
}
