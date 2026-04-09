use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Read a log file by path
    Read {
        /// Log file path (relative to logs/, e.g. "agent/completions/ac1-abc123.json")
        path: String,
    },
}

impl Commands {
    pub async fn handle(self, cli_config: &crate::Config) -> Result<crate::Output, crate::error::Error> {
        match self {
            Commands::Read { path } => {
                let client = objectiveai::filesystem::logs::LogsClient::new(cli_config.config_base_dir.as_deref());
                let content = client.read(&path).await?;
                Ok(crate::Output::LogsGet(content))
            }
        }
    }
}
