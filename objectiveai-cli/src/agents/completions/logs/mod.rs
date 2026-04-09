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
                Ok(crate::Output::LogsList(client.list_agent_completions(offset, limit)?))
            }
        }
    }
}
