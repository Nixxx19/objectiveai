use clap::{Args, Subcommand};
use futures::StreamExt;

/// How messages are provided to the agent completion.
#[derive(Args)]
#[group(required = true, multiple = false)]
pub struct MessageSource {
    /// Inline JSON messages array
    #[arg(long)]
    messages_inline: Option<String>,
    /// Inline Python code that produces the messages array
    #[arg(long)]
    messages_python_inline: Option<String>,
    /// Path to a Python file that produces the messages array
    #[arg(long)]
    messages_python_file: Option<std::path::PathBuf>,
}

impl MessageSource {
    fn resolve(self) -> Result<Vec<objectiveai::agent::completions::message::Message>, crate::error::Error> {
        if let Some(inline) = self.messages_inline {
            let mut de = serde_json::Deserializer::from_str(&inline);
            return serde_path_to_error::deserialize(&mut de)
                .map_err(crate::error::Error::PythonDeserialize);
        }
        if let Some(code) = self.messages_python_inline {
            return crate::python::exec_code(&code);
        }
        if let Some(path) = self.messages_python_file {
            return crate::python::exec_file(&path);
        }
        unreachable!("clap group ensures one is set")
    }
}

/// Agent args — supports mock remote via RemoteWithMock.
#[derive(Args)]
pub struct AgentArgs {
    /// Get agent by favorite name
    #[arg(long, conflicts_with_all = [
        "agent_remote", "agent_owner", "agent_repository", "agent_name", "agent_commit"
    ])]
    pub agent_favorite: Option<String>,
    /// Agent remote source (github, filesystem, or mock)
    #[arg(long, value_enum,
        requires_if("github", "agent_owner"),
        requires_if("github", "agent_repository"),
        requires_if("filesystem", "agent_owner"),
        requires_if("filesystem", "agent_repository"),
        requires_if("mock", "agent_name"),
    )]
    pub agent_remote: Option<crate::remote::RemoteWithMock>,
    /// Agent owner (github/filesystem)
    #[arg(long, conflicts_with = "agent_name")]
    pub agent_owner: Option<String>,
    /// Agent repository (github/filesystem)
    #[arg(long, conflicts_with = "agent_name")]
    pub agent_repository: Option<String>,
    /// Agent name (mock only)
    #[arg(long, conflicts_with_all = ["agent_owner", "agent_repository", "agent_commit"])]
    pub agent_name: Option<String>,
    /// Agent commit (optional, github/filesystem only)
    #[arg(long)]
    pub agent_commit: Option<String>,
}

impl AgentArgs {
    fn resolve(self) -> Result<objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional, crate::error::Error> {
        if let Some(fav_name) = self.agent_favorite {
            let (_, mut config) = crate::config::read()?;
            let favorites = config.agents().get_favorites().to_vec();
            let fav = favorites.into_iter().find(|f| f.get_name() == fav_name)
                .ok_or_else(|| crate::error::Error::FavoriteNotFound(fav_name))?;
            Ok(objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::Remote(fav.path.clone()))
        } else {
            let path = self.agent_remote
                .ok_or(crate::error::Error::MissingArgs("--agent-remote is required (or use --agent-favorite)"))?
                .into_path(self.agent_owner, self.agent_repository, self.agent_name, self.agent_commit)
                .ok_or(crate::error::Error::MissingArgs("--agent-owner and --agent-repository are required for github/filesystem, --agent-name for mock"))?;
            Ok(objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::Remote(path))
        }
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Standard agent completion
    Standard {
        #[command(flatten)]
        messages: MessageSource,
        #[command(flatten)]
        agent: AgentArgs,
        #[command(flatten)]
        continuation: crate::continuation::ContinuationArgs,
        #[command(flatten)]
        response_format: crate::response_format::ResponseFormatArgs,
        /// Seed for deterministic mock responses
        #[arg(long)]
        seed: Option<i64>,
    },
}

impl Commands {
    pub async fn handle(self) -> Result<crate::Output, crate::error::Error> {
        let (message_source, agent_args, continuation_args, response_format_args, seed) = match self {
            Commands::Standard { messages, agent, continuation, response_format, seed } => {
                (messages, agent, continuation, response_format, seed)
            }
        };

        let messages = message_source.resolve()?;
        let agent = agent_args.resolve()?;
        let continuation = continuation_args.resolve()?;
        let response_format = response_format_args.resolve()?
            .map(objectiveai::agent::completions::request::ResponseFormatParam::Single);

        let params = objectiveai::agent::completions::request::AgentCompletionCreateParams {
            messages,
            provider: None,
            agent,
            response_format,
            seed,
            stream: Some(true),
            continuation,
        };

        crate::api::run(|http_client| async move {
            let stream = objectiveai::agent::completions::create_agent_completion_streaming(
                &http_client, params,
            ).await?;
            tokio::pin!(stream);

            // Accumulate all chunks
            let mut accumulated: Option<objectiveai::agent::completions::response::streaming::AgentCompletionChunk> = None;
            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;
                match &mut accumulated {
                    Some(agg) => agg.push(&chunk),
                    None => accumulated = Some(chunk),
                }
            }

            let completion: objectiveai::agent::completions::response::unary::AgentCompletion =
                accumulated.ok_or(crate::error::Error::EmptyStream)?.into();

            // Extract the last assistant message content
            let content = completion.messages.iter().rev()
                .find_map(|msg| {
                    if let objectiveai::agent::completions::response::unary::Message::Assistant(asst) = msg {
                        asst.content.as_ref().map(|c| match c {
                            objectiveai::agent::completions::message::RichContent::Text(t) => t.clone(),
                            objectiveai::agent::completions::message::RichContent::Parts(parts) => {
                                parts.iter().filter_map(|p| match p {
                                    objectiveai::agent::completions::message::RichContentPart::Text { text } => Some(text.as_str()),
                                    _ => None,
                                }).collect::<Vec<_>>().join("")
                            }
                        })
                    } else {
                        None
                    }
                })
                .unwrap_or_default();

            Ok(content)
        }, true).await
    }
}
