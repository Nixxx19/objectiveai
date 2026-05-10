use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Get a message audio
    Get { id: String, message_index: u64, media_index: u64 },
    /// Subscribe to changes (wait for create/modify)
    Subscribe {
        id: String,
        message_index: u64,
        media_index: u64,
        #[arg(long)]
        require_modification: bool,
        timeout_ms: u64,
    },
    /// Clear all message audio
    Clear,
}

impl Commands {
    pub async fn handle(self, cli_config: &crate::Config, handle: &objectiveai_cli_lib::output::Handle) -> Result<(), crate::error::Error> {
        let client = objectiveai::filesystem::Client::new(cli_config.config_base_dir.as_deref(), None::<String>, None::<String>);
        match self {
            Commands::Get { id, message_index, media_index } => {
                let content = objectiveai::filesystem::logs::client::read_agent_completion_message_audio(&client, &id, message_index, media_index).await.map(objectiveai::filesystem::logs::LogContent::DataUrl)?;
                {
                crate::log_line::emit_log_content(content, handle).await;
                Ok(())
            }
            }
            Commands::Subscribe { id, message_index, media_index, timeout_ms, require_modification } => {
                let result = objectiveai::filesystem::logs::client::subscribe_agent_completion_message_audio(&client, &id, message_index, media_index, std::time::Duration::from_millis(timeout_ms), require_modification).await;
                {
                match result.map(objectiveai::filesystem::logs::LogContent::DataUrl) {
                    Some(content) => {
                        crate::log_line::emit_log_content(content, handle).await;
                        Ok(())
                    }
                    None => Err(crate::error::Error::LogSubscribeTimedOut),
                }
            }
            }
            Commands::Clear => {
                crate::log_line::emit_log_clear_count(objectiveai::filesystem::logs::client::clear_agent_completion_message_audio(&client).await?, handle).await;
                Ok(())
            },
        }
    }
}
