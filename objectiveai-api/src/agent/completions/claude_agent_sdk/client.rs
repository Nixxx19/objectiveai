use std::pin::Pin;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use futures::Stream;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::process::Command;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::LinesStream;

use super::super::{ContinuationItem, StreamItem, UpstreamClient};
use super::invention_server::InventionServer;
use super::prompt::Prompt;
use super::sdk_message::SDKMessage;

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
    ) -> impl Future<
        Output = Result<
            (Self::Stream, Self::State),
            objectiveai::error::ResponseError,
        >,
    > + Send
    + 'static {
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
                return Err(objectiveai::error::ResponseError::from(
                    &super::Error::InvalidByok,
                ));
            }

            validate_response_format(&agent.id, &params.response_format)
                .map_err(|e| objectiveai::error::ResponseError::from(&e))?;

            // Build prompt from messages + continuation (handles continuation validation).
            let prompt = Prompt::new(&messages, continuation.as_deref())
                .map_err(|e| objectiveai::error::ResponseError::from(&e))?;

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
            )
            .map_err(|e| objectiveai::error::ResponseError::from(&e))?;

            // Compute assistant_index from continuation.
            let assistant_index = continuation
                .as_deref()
                .map(|c| {
                    c.iter()
                        .filter(|item| {
                            matches!(
                                item,
                                ContinuationItem::State(_)
                                    | ContinuationItem::ToolMessage(_)
                            )
                        })
                        .count() as u64
                })
                .unwrap_or(0);

            let sdk_path = client.sdk_path().map(|s| s.to_owned());
            let tmp_id = client.next_id.fetch_add(1, Ordering::Relaxed);
            let agent_id = agent.id.clone();

            let initial_state = super::State {
                session_id: prompt.message.session_id.clone(),
            };

            let stream = async_stream::stream! {
                // Write JS to temp file.
                let tmp_dir = std::env::temp_dir();
                let tmp_path = tmp_dir.join(format!(
                    "claude_agent_sdk_{}_{tmp_id}.js",
                    std::process::id()
                ));
                if let Err(e) = std::fs::write(&tmp_path, &js) {
                    yield StreamItem::Chunk(
                        objectiveai::agent::completions::response::streaming::AgentCompletionChunk {
                            id: id.clone(),
                            error: Some(objectiveai::error::ResponseError::from(
                                &super::Error::Io(e.to_string()),
                            )),
                            ..Default::default()
                        },
                    );
                    return;
                }

                // Guard that removes the temp file on drop (handles early returns
                // and stream cancellation).
                struct TmpGuard(std::path::PathBuf);
                impl Drop for TmpGuard {
                    fn drop(&mut self) {
                        let _ = std::fs::remove_file(&self.0);
                    }
                }
                let _tmp_guard = TmpGuard(tmp_path.clone());

                // Spawn node subprocess.
                let mut cmd = Command::new("node");
                cmd.arg(&tmp_path)
                    .stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::piped())
                    .stderr(std::process::Stdio::piped());
                if let Some(ref sp) = sdk_path {
                    cmd.env("CLAUDE_AGENT_SDK_PATH", sp);
                }
                let mut child = match cmd.spawn() {
                    Ok(child) => child,
                    Err(e) => {
                        yield StreamItem::Chunk(
                            objectiveai::agent::completions::response::streaming::AgentCompletionChunk {
                                id: id.clone(),
                                error: Some(objectiveai::error::ResponseError::from(
                                    &super::Error::Spawn(e.to_string()),
                                )),
                                ..Default::default()
                            },
                        );
                        return;
                    }
                };

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

                let mut latest_session_id = String::new();
                let mut had_error = false;

                // Keep invention_server alive for the duration of the stream.
                let _invention_server_guard = invention_server;

                loop {
                    match lines_stream.next().await {
                        None => {
                            // Process ended — collect stderr.
                            let stderr_ctx = stderr_handle.await
                                .ok()
                                .unwrap_or_default();

                            if !stderr_ctx.is_empty() {
                                yield StreamItem::Chunk(
                                    objectiveai::agent::completions::response::streaming::AgentCompletionChunk {
                                        id: id.clone(),
                                        error: Some(objectiveai::error::ResponseError::from(
                                            &super::Error::Stderr(stderr_ctx.trim().to_owned()),
                                        )),
                                        ..Default::default()
                                    },
                                );
                                had_error = true;
                            }
                            break;
                        }
                        Some(Err(e)) => {
                            let _ = child.kill().await;
                            yield StreamItem::Chunk(
                                objectiveai::agent::completions::response::streaming::AgentCompletionChunk {
                                    id: id.clone(),
                                    error: Some(objectiveai::error::ResponseError::from(
                                        &super::Error::Io(e.to_string()),
                                    )),
                                    ..Default::default()
                                },
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
                                assistant_index,
                                is_byok,
                                cost_multiplier,
                            ) {
                                Some(Ok(chunk)) => {
                                    yield StreamItem::Chunk(chunk);
                                }
                                Some(Err(e)) => {
                                    yield StreamItem::Chunk(
                                        objectiveai::agent::completions::response::streaming::AgentCompletionChunk {
                                            id: id.clone(),
                                            error: Some(objectiveai::error::ResponseError::from(&e)),
                                            ..Default::default()
                                        },
                                    );
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
                    // Yield final state with session_id.
                    yield StreamItem::State(super::State {
                        session_id: latest_session_id,
                    });
                }
            };

            let boxed: Pin<Box<dyn Stream<Item = StreamItem<Self::State>> + Send>> =
                Box::pin(stream);
            Ok((boxed, initial_state))
        }
    }
}
