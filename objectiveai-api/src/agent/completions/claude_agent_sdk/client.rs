use std::pin::Pin;
use std::sync::Arc;
use futures::{Stream, StreamExt};
use indexmap::IndexMap;
use tokio::sync::OnceCell;

use super::super::{ContinuationItem, StreamItem, UpstreamClient};
use super::invention_server::InventionServer;
use super::mcp_server_config::{McpHttpServerConfig, McpServerConfig};
use super::prompt::Prompt;
use super::runner::{Runner, RunnerUpdate};
use super::sdk_message::SDKMessage;
use super::stdio::{RunParams, StdioEndStatus};
use crate::util::StreamOnce;

/// Claude Agent SDK client for agent completions.
///
/// Owns the Python runner subprocess for the lifetime of the
/// client. The subprocess is spawned **lazily** on the first
/// `create()` call and reused for every subsequent request — see
/// [`Client::runner_handle`]. The runner multiplexes N concurrent
/// streams over a single (stdin, stdout, stderr) triple, with the
/// in-flight cap enforced by `--query-limit`.
#[derive(Clone)]
pub struct Client {
    pub user_agent: String,
    pub enabled: bool,
    pub rate_limit_max_retries: u64,
    pub rate_limit_max_wait_secs: u64,
    /// FIFO concurrency cap forwarded to the runner via `--query-limit`.
    pub query_limit: u64,
    binary_path: Arc<std::sync::OnceLock<String>>,
    /// Lazily-spawned shared runner. Initialized on first request via
    /// `tokio::sync::OnceCell::get_or_try_init`. All concurrent
    /// `create()` callers race for the same singleton; only one
    /// initializer runs.
    runner: Arc<OnceCell<Arc<Runner>>>,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("user_agent", &self.user_agent)
            .field("enabled", &self.enabled)
            .field("rate_limit_max_retries", &self.rate_limit_max_retries)
            .field("rate_limit_max_wait_secs", &self.rate_limit_max_wait_secs)
            .field("query_limit", &self.query_limit)
            .field("runner_initialized", &self.runner.initialized())
            .finish()
    }
}

impl Client {
    pub fn new(
        user_agent: String,
        enabled: bool,
        rate_limit_max_retries: u64,
        rate_limit_max_wait_secs: u64,
        query_limit: u64,
    ) -> Self {
        Self {
            user_agent,
            enabled,
            rate_limit_max_retries,
            rate_limit_max_wait_secs,
            query_limit,
            binary_path: Arc::new(std::sync::OnceLock::new()),
            runner: Arc::new(OnceCell::new()),
        }
    }

    /// Extracts the embedded runner binary to a temp directory and returns its path.
    ///
    /// Cached after first extraction. Uses a content-based hash in the directory name
    /// so different API versions get separate binaries and the same version reuses
    /// the cached binary across restarts.
    ///
    /// Returns `None` when the crate is built without the
    /// `claude-agent-sdk` feature — in that configuration no runner
    /// binary is embedded, and `create()` returns `Error::NotEnabled`
    /// before this method is reached.
    #[cfg(feature = "claude-agent-sdk")]
    fn binary_path(&self) -> Option<&str> {
        let path = self.binary_path.get_or_init(|| {
            let binary = super::claude_agent_sdk_binary::CLAUDE_AGENT_SDK_RUNNER;

            // Fast fingerprint: hash length + head/tail for cache key.
            use std::hash::{Hash, Hasher};
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            binary.len().hash(&mut hasher);
            binary[..binary.len().min(4096)].hash(&mut hasher);
            binary[binary.len().saturating_sub(4096)..].hash(&mut hasher);
            let hash = hasher.finish();

            let binary_name = if cfg!(windows) {
                "objectiveai-claude-agent-sdk-runner.exe"
            } else {
                "objectiveai-claude-agent-sdk-runner"
            };

            let dir = std::env::temp_dir()
                .join(format!("objectiveai-sdk-runner-{hash:016x}"));
            let path = dir.join(binary_name);

            if !path.exists() {
                std::fs::create_dir_all(&dir).ok();
                if std::fs::write(&path, binary).is_err() {
                    return String::new();
                }
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = std::fs::set_permissions(
                        &path,
                        std::fs::Permissions::from_mode(0o755),
                    );
                }
            }

            path.to_string_lossy().to_string()
        });
        if path.is_empty() { None } else { Some(path.as_str()) }
    }

    #[cfg(not(feature = "claude-agent-sdk"))]
    fn binary_path(&self) -> Option<&str> {
        None
    }

    /// Get-or-init the shared runner subprocess. The first caller to
    /// hit this on a given `Client` pays the spawn cost; subsequent
    /// callers receive a clone of the same `Arc<Runner>`.
    async fn runner_handle(&self) -> Result<Arc<Runner>, super::Error> {
        let query_limit = self.query_limit;
        let binary_path = self
            .binary_path()
            .ok_or_else(|| {
                super::Error::Spawn(
                    "failed to extract claude-agent-sdk-runner binary".to_string(),
                )
            })?
            .to_owned();

        let runner = self
            .runner
            .get_or_try_init(|| async move {
                let r = Runner::spawn(&binary_path, query_limit)
                    .await
                    .map_err(|e| super::Error::Spawn(e.to_string()))?;
                Ok::<_, super::Error>(Arc::new(r))
            })
            .await?;
        Ok(runner.clone())
    }
}

/// Build the typed `mcp_servers` map that goes into [`RunParams`].
/// Replaces the old `serde_json::Value`-based intermediate
/// representation with strongly-typed [`McpServerConfig`].
fn build_mcp_servers(
    mcp_connections: &[Arc<crate::mcp::Connection>],
    invention_server: Option<&InventionServer>,
) -> IndexMap<String, McpServerConfig> {
    use std::collections::HashMap;

    let mut name_counts: HashMap<String, usize> = HashMap::new();
    for conn in mcp_connections {
        let name = &conn.initialize_result.server_info.name;
        *name_counts.entry(name.clone()).or_default() += 1;
    }

    let mut servers: IndexMap<String, McpServerConfig> = IndexMap::new();

    for conn in mcp_connections {
        let name = &conn.initialize_result.server_info.name;
        let key = if name_counts.get(name).copied().unwrap_or(0) > 1 {
            format!("{name} ({})", conn.url)
        } else {
            name.clone()
        };
        let config = McpHttpServerConfig::from(conn.as_ref());
        servers.insert(key, McpServerConfig::Http(config));
    }

    if let Some(inv) = invention_server {
        servers.insert(
            "objectiveai-invention".to_string(),
            McpServerConfig::Http(inv.mcp_server_config()),
        );
    }

    servers
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

/// Drop-guard that fires a best-effort `cancel` for `request_id` when
/// the consumer drops the returned stream early. Sending cancel for
/// a request that already finished is harmless — the runner replies
/// `end(error, "cancel-unknown-id")`, but we've already unregistered
/// from the runner's registry by then so the line is dropped.
struct CancelOnDrop {
    runner: Option<Arc<Runner>>,
    id: Option<String>,
}

impl CancelOnDrop {
    fn new(runner: Arc<Runner>, id: String) -> Self {
        Self {
            runner: Some(runner),
            id: Some(id),
        }
    }

    /// Disarm — the request finished naturally, no cancel needed.
    fn defuse(&mut self) {
        self.runner = None;
        self.id = None;
    }
}

impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        let (Some(runner), Some(id)) = (self.runner.take(), self.id.take()) else {
            return;
        };
        // Spawn rather than block — Drop is sync.
        if tokio::runtime::Handle::try_current().is_ok() {
            tokio::spawn(async move {
                let _ = runner.send_cancel(&id).await;
                runner.unregister(&id).await;
            });
        }
    }
}

impl UpstreamClient<objectiveai::agent::claude_agent_sdk::Agent, objectiveai::agent::claude_agent_sdk::Continuation> for Client {
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
        request_continuation: Option<&objectiveai::agent::claude_agent_sdk::Continuation>,
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
        _invention_type: Option<objectiveai::functions::inventions::prompts::StepPromptType>,
        _invention_step: Option<usize>,
        _invention_tasks_min: Option<u64>,
        _invention_input_schema: Option<String>,
    ) -> impl Future<
        Output = Result<
            Self::Stream,
            Self::Error,
        >,
    > + Send
    + 'static {
        let enabled = self.enabled;
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
        let request_continuation = request_continuation.cloned();
        let client = self.clone();

        async move {
            if !enabled {
                return Err(super::Error::NotEnabled);
            }

            // When built without the claude-agent-sdk feature, no
            // runner binary is embedded, so the client is non-functional
            // regardless of the `enabled` flag.
            #[cfg(not(feature = "claude-agent-sdk"))]
            {
                return Err(super::Error::NotEnabled);
            }

            if is_byok {
                return Err(super::Error::InvalidByok);
            }

            if !tools_enabled && has_tools {
                return Err(super::Error::ToolsNotAllowed);
            }

            validate_response_format(&agent.id, &params.response_format)?;

            // Build prompt from messages + continuation (handles continuation validation).
            let prompt = Prompt::new(&messages, continuation.as_deref(), request_continuation.as_ref())?;

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

            let mcp_servers = build_mcp_servers(
                &mcp_connections,
                invention_server.as_ref(),
            );

            // Compute assistant_index from continuation. State items
            // carry a message_count (may be >1 since the SDK handles
            // its own multi-turn loop). Other items count as 1.
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

            // Lazy-spawn (or reuse) the runner subprocess.
            let runner = client.runner_handle().await?;

            // Each agent-completions request gets its own caller-side
            // id. We use `id` (the upstream id) rather than minting a
            // separate UUID — the upstream id is already unique per
            // request and lets the runner's diag lines be cross-
            // referenced against agent-completion logs.
            let request_id = id.clone();

            let mut rx = runner
                .register(request_id.clone())
                .await
                .map_err(|e| super::Error::Spawn(e.to_string()))?;

            // Cancel-on-drop guard: arms now, defuses on natural
            // completion below.
            let mut cancel_guard = CancelOnDrop::new(runner.clone(), request_id.clone());

            // Build the params object — borrows from locals in this
            // async block, valid for the duration of the await.
            let session_id = prompt.message.session_id.as_str();
            let resume_arg: Option<&str> =
                if session_id.is_empty() { None } else { Some(session_id) };
            let user_agent_arg: Option<&str> =
                if client.user_agent.is_empty() { None } else { Some(client.user_agent.as_str()) };

            let run_params = RunParams {
                model: agent.base.model.as_str(),
                message: &prompt.message,
                system_prompt: prompt.system_prompt.as_deref(),
                effort: agent.base.effort,
                thinking_disabled: agent.base.thinking == Some(false),
                mcp_servers: &mcp_servers,
                resume: resume_arg,
                user_agent: user_agent_arg,
                rate_limit_max_retries: client.rate_limit_max_retries,
                rate_limit_max_wait_secs: client.rate_limit_max_wait_secs,
            };

            if let Err(e) = runner.send_run(&request_id, run_params).await {
                runner.unregister(&request_id).await;
                cancel_guard.defuse();
                return Err(super::Error::Spawn(e.to_string()));
            }

            let id_for_chunks = id.clone();
            let agent_id = agent.id.clone();

            let internal_stream = async_stream::stream! {
                // Keep invention server alive for the duration of the
                // stream and arm the cancel-on-drop.
                let _invention_server_guard = invention_server;
                let mut cancel_guard = cancel_guard;

                let mut latest_session_id = String::new();
                let mut had_error = false;
                let mut msg_index = assistant_index;
                // Most-recent assistant index, so the SDK's trailing
                // ResultMessage (a usage/cost summary, not a real
                // second turn) can re-use it. Per protocol, assistant
                // messages never sit back-to-back at distinct indices
                // — they alternate with tool messages — so the trailer
                // must merge into the assistant that just finished.
                let mut last_assistant_index: Option<u64> = None;

                loop {
                    let update = match rx.recv().await {
                        Some(u) => u,
                        None => {
                            // The runner unregistered us without sending
                            // an end (it does that when forwarding the
                            // `end` line). Treat as runner-died.
                            yield Err(super::Error::NoOutput);
                            had_error = true;
                            break;
                        }
                    };

                    match update {
                        RunnerUpdate::Event(sdk_msg) => {
                            // Track latest session_id.
                            if let Some(sid) = sdk_msg.session_id() {
                                if !sid.is_empty() {
                                    latest_session_id = sid.to_string();
                                }
                            }

                            // ResultMessage merges into the last
                            // assistant index instead of advancing.
                            let effective_index = match &sdk_msg {
                                SDKMessage::ResultMessage(_) => {
                                    last_assistant_index.unwrap_or(msg_index)
                                }
                                _ => msg_index,
                            };

                            match sdk_msg.into_downstream(
                                id_for_chunks.clone(),
                                created,
                                agent_id.clone(),
                                effective_index,
                                is_byok,
                                cost_multiplier,
                                objectiveai::agent::Upstream::ClaudeAgentSdk,
                            ) {
                                Some(Ok(chunk)) => {
                                    use objectiveai::agent::completions::response::streaming::MessageChunk;
                                    let mut advances_index = false;
                                    for m in &chunk.messages {
                                        match m {
                                            MessageChunk::Assistant(a) => {
                                                last_assistant_index = Some(a.index);
                                                if a.finish_reason.is_some() {
                                                    advances_index = true;
                                                }
                                            }
                                            MessageChunk::Tool(_) => {
                                                advances_index = true;
                                            }
                                        }
                                    }
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
                        RunnerUpdate::End(StdioEndStatus::Ok) => {
                            // Natural completion. Disarm the cancel
                            // guard — request is already done.
                            cancel_guard.defuse();
                            break;
                        }
                        RunnerUpdate::End(StdioEndStatus::Cancelled) => {
                            // Caller-initiated cancel landed. Defuse.
                            cancel_guard.defuse();
                            break;
                        }
                        RunnerUpdate::End(StdioEndStatus::Error { error }) => {
                            cancel_guard.defuse();
                            yield Err(super::Error::Stderr(error));
                            had_error = true;
                            break;
                        }
                        RunnerUpdate::Diag { level: _, message: _ } => {
                            // Diags are informational (rate-limit
                            // retries etc.) — no downstream channel
                            // for them at this layer. Drop them; the
                            // user-visible signal is the eventual
                            // event/end.
                        }
                        RunnerUpdate::Fatal(message) => {
                            cancel_guard.defuse();
                            yield Err(super::Error::Stderr(message));
                            had_error = true;
                            break;
                        }
                        RunnerUpdate::RunnerExited => {
                            cancel_guard.defuse();
                            yield Err(super::Error::NoOutput);
                            had_error = true;
                            break;
                        }
                    }
                }

                if !had_error {
                    yield Ok(StreamItem::State(super::State {
                        session_id: latest_session_id,
                        message_count: msg_index - assistant_index,
                    }));
                }
            };

            // Await the first stream item. If it is an error,
            // return Err so the caller never sees an error as the
            // first yielded item (per the upstream contract).
            let mut stream = Box::pin(internal_stream);
            match stream.next().await {
                Some(Err(e)) => Err(e),
                Some(Ok(first)) => {
                    let id_for_stream = id.clone();
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
                    Ok(boxed)
                }
                None => Err(super::Error::NoOutput),
            }
        }
    }

    fn response_continuation(
        &self,
        mcp_sessions: indexmap::IndexMap<String, String>,
        request_continuation: Option<&objectiveai::agent::claude_agent_sdk::Continuation>,
        _messages: &[objectiveai::agent::completions::message::Message],
        continuation: Option<&[ContinuationItem<Self::State>]>,
    ) -> objectiveai::agent::claude_agent_sdk::Continuation {
        // Extract session_id from last State in continuation, fall back to request continuation.
        let session_id = continuation
            .and_then(|items| {
                items.iter().rev().find_map(|item| match item {
                    ContinuationItem::State(state) => {
                        if state.session_id.is_empty() { None } else { Some(state.session_id.clone()) }
                    }
                    _ => None,
                })
            })
            .or_else(|| request_continuation.map(|rc| rc.session_id.clone()))
            .unwrap_or_default();

        objectiveai::agent::claude_agent_sdk::Continuation {
            upstream: objectiveai::agent::claude_agent_sdk::Upstream::default(),
            session_id,
            mcp_sessions,
        }
    }
}
