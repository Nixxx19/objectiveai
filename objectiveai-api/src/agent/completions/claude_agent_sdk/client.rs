use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use futures::{Stream, StreamExt};
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::process::Command;
use tokio_stream::wrappers::LinesStream;

use super::super::{ContinuationItem, StreamItem, UpstreamClient};
use super::invention_server::InventionServer;
use super::prompt::Prompt;
use super::sdk_message::SDKMessage;
use crate::util::StreamOnce;

/// Claude Agent SDK client for agent completions.
///
/// Lazily resolves the path to the globally installed `@anthropic-ai/claude-agent-sdk`
/// package and passes it to spawned Node.js subprocesses via an environment variable.
#[derive(Debug, Clone)]
pub struct Client {
    pub user_agent: Option<String>,
    sdk_path: Arc<std::sync::OnceLock<String>>,
    next_id: Arc<AtomicU64>,
}

impl Client {
    pub fn new(user_agent: Option<String>) -> Self {
        Self {
            user_agent,
            sdk_path: Arc::new(std::sync::OnceLock::new()),
            next_id: Arc::new(AtomicU64::new(0)),
        }
    }

    /// Resolves the absolute path to the `@anthropic-ai/claude-agent-sdk` package.
    ///
    /// Cached after first resolution. Uses `node -e` to call `require.resolve`.
    fn sdk_path(&self) -> Option<&str> {
        let path = self.sdk_path.get_or_init(|| {
            std::process::Command::new("node")
                .arg("-e")
                .arg("console.log(require.resolve('@anthropic-ai/claude-agent-sdk'))")
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::null())
                .output()
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
                .unwrap_or_default()
        });
        if path.is_empty() { None } else { Some(path.as_str()) }
    }
}

/// Validates that the response_format is compatible with the Claude Agent SDK.
///
/// Only `None` or `Text` formats are supported.
fn validate_response_format(
    agent_id: &str,
    response_format: &Option<objectiveai::agent::completions::request::ResponseFormatParam>,
) -> Result<(), super::Error> {
    use objectiveai::agent::completions::request::{ResponseFormat, ResponseFormatParam};

    match response_format {
        None => Ok(()),
        Some(ResponseFormatParam::Single(ResponseFormat::Text)) => Ok(()),
        Some(ResponseFormatParam::PerAgent(map)) => {
            match map.get(agent_id) {
                None => Ok(()),
                Some(ResponseFormat::Text) => Ok(()),
                Some(_) => Err(super::Error::UnsupportedResponseFormat),
            }
        }
        Some(_) => Err(super::Error::UnsupportedResponseFormat),
    }
}

impl UpstreamClient<objectiveai::agent::claude_agent_sdk::Agent> for Client {
    type State = super::State;
    type Stream = Pin<
        Box<dyn Stream<Item = StreamItem<Self::State>> + Send + 'static>,
    >;
    type Error = super::Error;

    #[allow(unused_variables)]
    fn create(
        &self,
        id: &str,
        created: u64,
        agent: &objectiveai::agent::claude_agent_sdk::Agent,
        params: &objectiveai::agent::completions::request::AgentCompletionCreateParams,
        messages: &[objectiveai::agent::completions::message::Message],
        mcp_connections: &[Arc<crate::mcp::Connection>],
        invention_tools: Option<
            &[objectiveai::functions::inventions::InventionTool],
        >,
        tool_names: &[String],
        tool_map: &std::collections::HashMap<String, super::super::tool::ResolvedTool>,
        continuation: Option<&[ContinuationItem<Self::State>]>,
        byok: Option<&str>,
        cost_multiplier: rust_decimal::Decimal,
        _tools_enabled: bool,
    ) -> impl Future<
        Output = Result<
            (Self::Stream, Self::State),
            Self::Error,
        >,
    > + Send
    + 'static {
        let tools_enabled = _tools_enabled;
        let has_tools = !tool_names.is_empty();
        let is_byok = byok.is_some();
        let id = id.to_string();
        let agent = agent.clone();
        let params = params.clone();
        let messages = messages.to_vec();
        let mcp_connections = mcp_connections.to_vec();
        let invention_tools = invention_tools.map(|t| t.to_vec());
        let continuation = continuation.map(|c| c.to_vec());
        let client = self.clone();

        async move {
            if is_byok {
                return Err(super::Error::InvalidByok);
            }

            if !tools_enabled && has_tools {
                return Err(super::Error::ToolsNotAllowed);
            }

            validate_response_format(&agent.id, &params.response_format)?;

            // Build prompt from messages + continuation (handles continuation validation).
            let prompt = Prompt::new(&messages, continuation.as_deref())?;

            // Spawn invention server if invention tools are provided.
            let invention_server = if let Some(ref tools) = invention_tools {
                if !tools.is_empty() {
                    Some(InventionServer::new(tools.clone()).await)
                } else {
                    None
                }
            } else {
                None
            };

            // Build JS code.
            let js = super::js::build_js(
                &prompt,
                &agent.base.model,
                agent.base.effort,
                agent.base.thinking,
                &mcp_connections,
                invention_server.as_ref(),
                client.user_agent.as_deref(),
            )?;

            // Compute assistant_index from continuation.
            // State items carry a message_count (may be >1 since the SDK
            // handles its own multi-turn loop). Other items count as 1.
            let assistant_index = continuation
                .as_deref()
                .map(|c| {
                    c.iter()
                        .map(|item| match item {
                            ContinuationItem::State(s) => s.message_count,
                            ContinuationItem::ToolMessage(_) => 1,
                            ContinuationItem::UserMessage(_) => 0,
                        })
                        .sum::<u64>()
                })
                .unwrap_or(0);

            let sdk_path = client.sdk_path().map(|s| s.to_owned());
            let tmp_id = client.next_id.fetch_add(1, Ordering::Relaxed);
            let agent_id = agent.id.clone();

            let initial_state = super::State {
                session_id: prompt.message.session_id.clone(),
                message_count: 0,
            };

            // Write JS to temp file.
            let tmp_dir = std::env::temp_dir();
            let tmp_path = tmp_dir.join(format!(
                "claude_agent_sdk_{}_{tmp_id}.js",
                std::process::id()
            ));
            std::fs::write(&tmp_path, &js).map_err(|e| {
                super::Error::Io(e.to_string())
            })?;

            // Guard that removes the temp file on drop (handles early returns
            // and stream cancellation).
            struct TmpGuard(std::path::PathBuf);
            impl Drop for TmpGuard {
                fn drop(&mut self) {
                    let _ = std::fs::remove_file(&self.0);
                }
            }
            let tmp_guard = TmpGuard(tmp_path.clone());

            // Spawn node subprocess.
            let mut cmd = Command::new("node");
            cmd.arg(&tmp_path)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped());
            if let Some(ref sp) = sdk_path {
                cmd.env("CLAUDE_AGENT_SDK_PATH", sp);
            }
            let mut child = cmd.spawn().map_err(|e| {
                super::Error::Spawn(e.to_string())
            })?;

            // Collect stderr in background.
            let stderr = child.stderr.take().expect("stderr was piped");
            let stderr_handle = tokio::spawn(async move {
                let mut buf = String::new();
                let mut reader = BufReader::new(stderr);
                let _ = tokio::io::AsyncReadExt::read_to_string(&mut reader, &mut buf).await;
                buf
            });

            // Read stdout lines.
            let stdout = child.stdout.take().expect("stdout was piped");
            let reader = BufReader::new(stdout);
            let mut lines_stream = LinesStream::new(reader.lines());

            let id_for_peek = id.clone();
            let internal_stream = async_stream::stream! {
                // Keep guards alive for the duration of the stream.
                let _tmp_guard = tmp_guard;
                let _invention_server_guard = invention_server;

                let mut latest_session_id = String::new();
                let mut had_error = false;
                let mut msg_index = assistant_index;

                loop {
                    match lines_stream.next().await {
                        None => {
                            // Process ended — collect stderr.
                            let stderr_ctx = stderr_handle.await
                                .ok()
                                .unwrap_or_default();

                            if !stderr_ctx.is_empty() {
                                yield Err(
                                    super::Error::Stderr(stderr_ctx.trim().to_owned()),
                                );
                                had_error = true;
                            }
                            break;
                        }
                        Some(Err(e)) => {
                            let _ = child.kill().await;
                            yield Err(
                                super::Error::Io(e.to_string()),
                            );
                            had_error = true;
                            break;
                        }
                        Some(Ok(line)) => {
                            let trimmed = line.trim();
                            if trimmed.is_empty() {
                                continue;
                            }

                            let sdk_msg: SDKMessage = match serde_json::from_str(trimmed) {
                                Ok(msg) => msg,
                                Err(e) => {
                                    // Log deserialization errors but continue — unknown
                                    // message types are expected as the SDK evolves.
                                    continue;
                                }
                            };

                            // Track latest session_id.
                            if let Some(sid) = sdk_msg.session_id() {
                                if !sid.is_empty() {
                                    latest_session_id = sid.to_string();
                                }
                            }

                            match sdk_msg.into_downstream(
                                id.clone(),
                                created,
                                agent_id.clone(),
                                msg_index,
                                is_byok,
                                cost_multiplier,
                            ) {
                                Some(Ok(chunk)) => {
                                    // Advance the index when a message slot is
                                    // complete: an assistant turn with a finish
                                    // reason, or a tool response.
                                    use objectiveai::agent::completions::response::streaming::MessageChunk;
                                    let advances_index = chunk.messages.iter().any(|m| match m {
                                        MessageChunk::Assistant(a) => a.finish_reason.is_some(),
                                        MessageChunk::Tool(_) => true,
                                    });
                                    yield Ok(StreamItem::Chunk(chunk));
                                    if advances_index {
                                        msg_index += 1;
                                    }
                                }
                                Some(Err(e)) => {
                                    yield Err(e);
                                    had_error = true;
                                    break;
                                }
                                None => {
                                    // Ignored message type.
                                }
                            }
                        }
                    }
                }

                if !had_error {
                    // Yield final state with session_id and message count.
                    yield Ok(StreamItem::State(super::State {
                        session_id: latest_session_id,
                        message_count: msg_index - assistant_index,
                    }));
                }
            };

            // Await the first stream item. If it is an error,
            // return Err so the caller never sees an error as the
            // first yielded item.
            let mut stream = Box::pin(internal_stream);
            match stream.next().await {
                Some(Err(e)) => {
                    return Err(e);
                }
                Some(Ok(first)) => {
                    // Map the remaining internal stream: typed errors become
                    // error chunks for mid-stream delivery to the client.
                    let id_for_stream = id_for_peek.clone();
                    let rest = stream.map(move |item| match item {
                        Ok(si) => si,
                        Err(e) => {
                            use objectiveai::error::StatusError;
                            StreamItem::Chunk(
                                objectiveai::agent::completions::response::streaming::AgentCompletionChunk {
                                    id: id_for_stream.clone(),
                                    error: Some(objectiveai::error::ResponseError {
                                        code: e.status(),
                                        message: e.message()
                                            .unwrap_or(serde_json::Value::Null),
                                    }),
                                    ..Default::default()
                                },
                            )
                        }
                    });
                    let boxed: Pin<Box<dyn Stream<Item = StreamItem<Self::State>> + Send>> =
                        Box::pin(StreamOnce::new(first).chain(rest));
                    Ok((boxed, initial_state))
                }
                None => {
                    return Err(super::Error::NoOutput);
                }
            }
        }
    }
}
