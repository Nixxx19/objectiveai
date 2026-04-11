use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Get a continuation log
    Get { id: String },
    /// Subscribe to changes (wait for create/modify)
    Subscribe {
        id: String,
        #[arg(long)]
        require_modification: bool,
        timeout_ms: u64,
    },
    /// Clear all continuation logs
    Clear,
}

impl Commands {
    pub async fn handle(self, cli_config: &crate::Config) -> Result<crate::Output, crate::error::Error> {
        let client = objectiveai::filesystem::Client::new(cli_config.config_base_dir.as_deref(), None::<String>, None::<String>);
        match self {
            Commands::Get { id } => {
                let content = objectiveai::filesystem::logs::client::read_agent_completion_continuation(&client, &id).await.map(objectiveai::filesystem::logs::LogContent::Json)?;
                Ok(crate::Output::LogsGet(content))
            }
            Commands::Subscribe { id, timeout_ms, require_modification } => {
                let result = objectiveai::filesystem::logs::client::subscribe_agent_completion_continuation(&client, &id, std::time::Duration::from_millis(timeout_ms), require_modification).await;
                Ok(crate::Output::LogsSubscribe(result.map(objectiveai::filesystem::logs::LogContent::Json)))
            }
            Commands::Clear => Ok(crate::Output::LogsClear(objectiveai::filesystem::logs::client::clear_agent_completion_continuations(&client).await?)),
        }
    }
}
