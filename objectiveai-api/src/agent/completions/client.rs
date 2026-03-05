use std::{sync::Arc, time::Duration};
use crate::{ctx, util::StreamOnce};

pub fn response_id(created: u64) -> String {
    let uuid = uuid::Uuid::new_v4();
    format!("agtcpl-{}-{created}", uuid.simple())
}

pub struct Client<CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, FAGENT, CUSG> {
    /// MCP Client
    pub mcp_client: Arc<crate::mcp::Client>,
    /// Caching fetcher for Ensemble LLM definitions.
    pub agent_fetcher:
        Arc<super::super::fetcher::CachingFetcher<CTXEXT, FAGENT>>,
    /// Handler for tracking usage after completion.
    pub usage_handler: Arc<CUSG>,
    /// Upstream client for Openrouter agents.
    pub openrouter: Arc<OPENROUTER>,
    /// Upstream client for Claude Agent SDK agents.
    pub claude_agent_sdk: Arc<CLAUDEAGENTSDK>,
    /// Upstream client for Mock agents.
    pub mock: Arc<MOCK>,

    /// Current backoff interval for retry logic.
    pub backoff_current_interval: Duration,
    /// Initial backoff interval for retry logic.
    pub backoff_initial_interval: Duration,
    /// Randomization factor for backoff jitter.
    pub backoff_randomization_factor: f64,
    /// Multiplier for exponential backoff growth.
    pub backoff_multiplier: f64,
    /// Maximum backoff interval.
    pub backoff_max_interval: Duration,
    /// Maximum total time to spend on retries.
    pub backoff_max_elapsed_time: Duration,
    /// Maximum wait time for the first chunk in a streaming response.
    pub first_chunk_timeout: Duration,
    /// Maximum wait time between subsequent chunks in a streaming response.
    pub other_chunk_timeout: Duration,
}

impl<CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, FAGENT, CUSG> Client<CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, FAGENT, CUSG> {
    pub fn new(
        mcp_client: Arc<crate::mcp::Client>,
        agent_fetcher: Arc<super::super::fetcher::CachingFetcher<CTXEXT, FAGENT>>,
        usage_handler: Arc<CUSG>,
        openrouter: Arc<OPENROUTER>,
        claude_agent_sdk: Arc<CLAUDEAGENTSDK>,
        mock: Arc<MOCK>,
        backoff_current_interval: Duration,
        backoff_initial_interval: Duration,
        backoff_randomization_factor: f64,
        backoff_multiplier: f64,
        backoff_max_interval: Duration,
        backoff_max_elapsed_time: Duration,
        first_chunk_timeout: Duration,
        other_chunk_timeout: Duration,
    ) -> Self {
        Self {
            mcp_client,
            agent_fetcher,
            usage_handler,
            openrouter,
            claude_agent_sdk,
            mock,
            backoff_current_interval,
            backoff_initial_interval,
            backoff_randomization_factor,
            backoff_multiplier,
            backoff_max_interval,
            backoff_max_elapsed_time,
            first_chunk_timeout,
            other_chunk_timeout,
        }
    }
}

impl<CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, FAGENT, CUSG> Clone
    for Client<CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, FAGENT, CUSG>
{
    fn clone(&self) -> Self {
        Self {
            mcp_client: self.mcp_client.clone(),
            agent_fetcher: self.agent_fetcher.clone(),
            usage_handler: self.usage_handler.clone(),
            openrouter: self.openrouter.clone(),
            claude_agent_sdk: self.claude_agent_sdk.clone(),
            mock: self.mock.clone(),
            backoff_current_interval: self.backoff_current_interval,
            backoff_initial_interval: self.backoff_initial_interval,
            backoff_randomization_factor: self.backoff_randomization_factor,
            backoff_multiplier: self.backoff_multiplier,
            backoff_max_interval: self.backoff_max_interval,
            backoff_max_elapsed_time: self.backoff_max_elapsed_time,
            first_chunk_timeout: self.first_chunk_timeout,
            other_chunk_timeout: self.other_chunk_timeout,
        }
    }
}

impl<CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, FAGENT, CUSG> Client<CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, FAGENT, CUSG>
where
    CTXEXT: ctx::ContextExt + Send + Sync + 'static,
    OPENROUTER: super::UpstreamClient<objectiveai::agent::openrouter::Agent> + Send + Sync + 'static,
    CLAUDEAGENTSDK: super::UpstreamClient<objectiveai::agent::claude_agent_sdk::Agent> + Send + Sync + 'static,
    MOCK: super::UpstreamClient<objectiveai::agent::mock::Agent> + Send + Sync + 'static,
    FAGENT: super::super::fetcher::Fetcher<CTXEXT> + Send + Sync + 'static,
    CUSG: super::usage_handler::UsageHandler<CTXEXT> + Send + Sync + 'static,
{
    pub async fn create_streaming(
        &self,
        ctx: ctx::Context<CTXEXT>,
        params: Arc<objectiveai::agent::completions::request::AgentCompletionCreateParams>,
        continuation: Option<
            super::Continuation<
                OPENROUTER::State,
                CLAUDEAGENTSDK::State,
                MOCK::State,
            >,
        >,
        invention_tools: Option<Vec<objectiveai::functions::inventions::InventionTool>>,
    ) -> Result<
        impl futures::Stream<
            Item = super::StreamItem<
                super::Continuation<
                    OPENROUTER::State,
                    CLAUDEAGENTSDK::State,
                    MOCK::State,
                >,
            >,
        >,
        super::Error,
    > {
        let created = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let id = response_id(created);

        // backoff and timeouts
        let backoff = backoff::ExponentialBackoff {
            current_interval: self.backoff_current_interval,
            initial_interval: self.backoff_initial_interval,
            randomization_factor: self.backoff_randomization_factor,
            multiplier: self.backoff_multiplier,
            max_interval: self.backoff_max_interval,
            start_time: std::time::Instant::now(),
            max_elapsed_time: Some(self.backoff_max_elapsed_time),
            clock: backoff::SystemClock::default(),
        };
        let first_chunk_timeout = self.first_chunk_timeout;
        let other_chunk_timeout = self.other_chunk_timeout;

        // Placeholder: return an empty stream.
        Ok(futures::stream::empty())
    }

    /// Resolves agents and connects to their MCP servers concurrently.
    ///
    /// If `continuation` is provided, returns the agent and MCP connections
    /// stored in it directly (single-element vec, no spawned tasks).
    ///
    /// Otherwise, for each agent in `params` (primary + fallbacks), spawns a
    /// tokio task that calls [`resolve_agent`](Self::resolve_agent). Returns
    /// `Ok(None)` for agents that are skipped (continuation mismatch or
    /// missing MCP authorization).
    pub fn resolve_agents(
        &self,
        ctx: ctx::Context<CTXEXT>,
        params: &objectiveai::agent::completions::request::AgentCompletionCreateParams,
        continuation: Option<
            &super::Continuation<
                OPENROUTER::State,
                CLAUDEAGENTSDK::State,
                MOCK::State,
            >,
        >,
    ) -> Vec<
        tokio::task::JoinHandle<
            Result<
                Option<(objectiveai::agent::Agent, Vec<Arc<crate::mcp::Connection>>)>,
                super::Error,
            >,
        >,
    > {
        // If continuation is provided, return its agent and connections directly.
        if let Some(cont) = continuation {
            let (agent, mcp_connections) = match cont {
                super::Continuation::Openrouter { agent, mcp_connections, .. } => {
                    (objectiveai::agent::Agent::Openrouter(agent.clone()), mcp_connections.clone())
                }
                super::Continuation::ClaudeAgentSdk { agent, mcp_connections, .. } => {
                    (objectiveai::agent::Agent::ClaudeAgentSdk(agent.clone()), mcp_connections.clone())
                }
                super::Continuation::Mock { agent, mcp_connections, .. } => {
                    (objectiveai::agent::Agent::Mock(agent.clone()), mcp_connections.clone())
                }
            };
            return vec![tokio::spawn(async move {
                Ok(Some((agent, mcp_connections)))
            })];
        }

        let request_agents = std::iter::once(&params.agent)
            .chain(params.agents.iter().flatten());

        let mcp_server_authorization = params.mcp_server_authorization.clone();
        let mut handles = Vec::new();

        for request_agent in request_agents {
            let request_agent = request_agent.clone();
            let ctx = ctx.clone();
            let client = self.clone();
            let mcp_server_authorization = mcp_server_authorization.clone();

            handles.push(tokio::spawn(async move {
                client
                    .resolve_agent(
                        ctx,
                        &request_agent,
                        mcp_server_authorization.as_ref(),
                        None,
                    )
                    .await
            }));
        }

        handles
    }

    /// Resolves a request agent (inline or by ID) into a validated Agent
    /// and connects to its MCP servers.
    ///
    /// Returns `Ok(None)` if:
    /// - The agent's upstream kind doesn't match the continuation
    /// - An MCP server requires authorization but none was provided
    pub async fn resolve_agent(
        &self,
        ctx: ctx::Context<CTXEXT>,
        agent: &objectiveai::agent::completions::request::Agent,
        mcp_server_authorization: Option<&indexmap::IndexMap<String, String>>,
        continuation: Option<objectiveai::agent::Upstream>,
    ) -> Result<
        Option<(objectiveai::agent::Agent, Vec<Arc<crate::mcp::Connection>>)>,
        super::Error,
    > {
        use objectiveai::agent::completions::request::Agent as RequestAgent;

        let agent = match agent {
            RequestAgent::Provided(base) => {
                // Check upstream kind before validation so that a
                // continuation mismatch returns None instead of an error.
                if let Some(expected) = continuation {
                    if base.upstream() != expected {
                        return Ok(None);
                    }
                }
                objectiveai::agent::Agent::try_from(base.clone())
                    .map_err(super::Error::InvalidAgent)?
            }
            RequestAgent::Id(id) => {
                match self.agent_fetcher.fetch(ctx, id).await? {
                    Some((agent, _created)) => {
                        if let Some(expected) = continuation {
                            if agent.base().upstream() != expected {
                                return Ok(None);
                            }
                        }
                        agent
                    }
                    None => return Err(super::Error::AgentNotFound(id.clone())),
                }
            }
        };

        // Connect to MCP servers concurrently.
        let mcp_connections = match agent.base().mcp_servers() {
            Some(servers) if !servers.is_empty() => {
                let mut futs = Vec::with_capacity(servers.len());
                for server in servers {
                    let authorization = if server.authorization {
                        match mcp_server_authorization.and_then(|m| m.get(&server.url)) {
                            Some(auth) => Some(auth.clone()),
                            None => return Ok(None),
                        }
                    } else {
                        None
                    };
                    futs.push(self.mcp_client.connect(server.url.clone(), authorization));
                }

                let results = futures::future::join_all(futs).await;
                let mut connections = Vec::with_capacity(results.len());
                for result in results {
                    connections.push(result?);
                }
                connections
            }
            _ => Vec::new(),
        };

        Ok(Some((agent, mcp_connections)))
    }
}
