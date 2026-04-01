use clap::Args;

/// Shared continuation arguments for commands that support multi-turn conversations.
#[derive(Args)]
pub struct ContinuationArgs {
    // --- From response (raw base64, any upstream) ---

    /// OpenRouter continuation from a previous response (base64-encoded).
    #[arg(long, group = "continuation")]
    pub openrouter_continuation_from_response: Option<String>,

    /// Claude Agent SDK continuation from a previous response (base64-encoded).
    #[arg(long, group = "continuation")]
    pub claude_agent_sdk_continuation_from_response: Option<String>,

    /// Mock continuation from a previous response (base64-encoded).
    #[arg(long, group = "continuation")]
    pub mock_continuation_from_response: Option<String>,

    // --- OpenRouter messages ---

    /// OpenRouter continuation messages as inline JSON.
    #[arg(long, group = "continuation")]
    pub openrouter_continuation_messages_inline: Option<String>,

    /// OpenRouter continuation messages from inline Python code.
    #[arg(long, group = "continuation")]
    pub openrouter_continuation_messages_python_inline: Option<String>,

    /// OpenRouter continuation messages from a Python file.
    #[arg(long, group = "continuation")]
    pub openrouter_continuation_messages_python_file: Option<std::path::PathBuf>,

    // --- Mock messages ---

    /// Mock continuation messages as inline JSON.
    #[arg(long, group = "continuation")]
    pub mock_continuation_messages_inline: Option<String>,

    /// Mock continuation messages from inline Python code.
    #[arg(long, group = "continuation")]
    pub mock_continuation_messages_python_inline: Option<String>,

    /// Mock continuation messages from a Python file.
    #[arg(long, group = "continuation")]
    pub mock_continuation_messages_python_file: Option<std::path::PathBuf>,

    // --- Claude Agent SDK session ---

    /// Claude Agent SDK continuation with a session ID
    /// (the UUID from tool result paths, e.g. ~/.claude/projects/{project}/{session-id}/tool-results/).
    #[arg(long, group = "continuation")]
    pub claude_agent_sdk_continuation_session_id: Option<String>,
}

impl ContinuationArgs {
    /// Resolves the continuation arguments into a base64-encoded continuation string,
    /// or None if no continuation was provided.
    pub fn resolve(self) -> Result<Option<String>, crate::error::Error> {
        // From response — already base64-encoded, pass through.
        if let Some(s) = self.openrouter_continuation_from_response {
            return Ok(Some(s));
        }
        if let Some(s) = self.claude_agent_sdk_continuation_from_response {
            return Ok(Some(s));
        }
        if let Some(s) = self.mock_continuation_from_response {
            return Ok(Some(s));
        }

        // OpenRouter messages
        if let Some(inline) = self.openrouter_continuation_messages_inline {
            let messages: Vec<objectiveai::agent::completions::message::Message> = {
                let mut de = serde_json::Deserializer::from_str(&inline);
                serde_path_to_error::deserialize(&mut de)
                    .map_err(crate::error::Error::PythonDeserialize)?
            };
            let cont = objectiveai::agent::Continuation::Openrouter(
                objectiveai::agent::openrouter::Continuation {
                    upstream: objectiveai::agent::openrouter::Upstream::Openrouter,
                    messages,
                    mcp_sessions: Default::default(),
                },
            );
            return Ok(Some(cont.to_string()));
        }
        if let Some(code) = self.openrouter_continuation_messages_python_inline {
            let messages: Vec<objectiveai::agent::completions::message::Message> =
                crate::python::exec_code(&code)?;
            let cont = objectiveai::agent::Continuation::Openrouter(
                objectiveai::agent::openrouter::Continuation {
                    upstream: objectiveai::agent::openrouter::Upstream::Openrouter,
                    messages,
                    mcp_sessions: Default::default(),
                },
            );
            return Ok(Some(cont.to_string()));
        }
        if let Some(path) = self.openrouter_continuation_messages_python_file {
            let messages: Vec<objectiveai::agent::completions::message::Message> =
                crate::python::exec_file(&path)?;
            let cont = objectiveai::agent::Continuation::Openrouter(
                objectiveai::agent::openrouter::Continuation {
                    upstream: objectiveai::agent::openrouter::Upstream::Openrouter,
                    messages,
                    mcp_sessions: Default::default(),
                },
            );
            return Ok(Some(cont.to_string()));
        }

        // Mock messages
        if let Some(inline) = self.mock_continuation_messages_inline {
            let messages: Vec<objectiveai::agent::completions::message::Message> = {
                let mut de = serde_json::Deserializer::from_str(&inline);
                serde_path_to_error::deserialize(&mut de)
                    .map_err(crate::error::Error::PythonDeserialize)?
            };
            let cont = objectiveai::agent::Continuation::Mock(
                objectiveai::agent::mock::Continuation {
                    upstream: objectiveai::agent::mock::Upstream::Mock,
                    messages,
                    mcp_sessions: Default::default(),
                },
            );
            return Ok(Some(cont.to_string()));
        }
        if let Some(code) = self.mock_continuation_messages_python_inline {
            let messages: Vec<objectiveai::agent::completions::message::Message> =
                crate::python::exec_code(&code)?;
            let cont = objectiveai::agent::Continuation::Mock(
                objectiveai::agent::mock::Continuation {
                    upstream: objectiveai::agent::mock::Upstream::Mock,
                    messages,
                    mcp_sessions: Default::default(),
                },
            );
            return Ok(Some(cont.to_string()));
        }
        if let Some(path) = self.mock_continuation_messages_python_file {
            let messages: Vec<objectiveai::agent::completions::message::Message> =
                crate::python::exec_file(&path)?;
            let cont = objectiveai::agent::Continuation::Mock(
                objectiveai::agent::mock::Continuation {
                    upstream: objectiveai::agent::mock::Upstream::Mock,
                    messages,
                    mcp_sessions: Default::default(),
                },
            );
            return Ok(Some(cont.to_string()));
        }

        // Claude Agent SDK session ID
        if let Some(session_id) = self.claude_agent_sdk_continuation_session_id {
            let cont = objectiveai::agent::Continuation::ClaudeAgentSdk(
                objectiveai::agent::claude_agent_sdk::Continuation {
                    upstream: objectiveai::agent::claude_agent_sdk::Upstream::ClaudeAgentSdk,
                    session_id,
                    mcp_sessions: Default::default(),
                },
            );
            return Ok(Some(cont.to_string()));
        }

        Ok(None)
    }
}
