use std::{collections::HashMap, pin::Pin, sync::Arc, time::Duration};

use crate::ctx;
use futures::StreamExt;

/// A function that transforms messages before they are sent to an upstream.
/// Keyed by agent ID so each agent in an swarm can receive different messages.
pub type TransformMessages = HashMap<
    String,
    Box<dyn Fn(Vec<objectiveai::agent::completions::message::Message>) -> Vec<objectiveai::agent::completions::message::Message> + Send + Sync>,
>;

pub fn response_id(created: u64) -> String {
    let uuid = uuid::Uuid::new_v4();
    format!("agtcpl-{}-{created}", uuid.simple())
}

// ---------------------------------------------------------------------------

/// A shared, re-awaitable handle to an agent's MCP connections.
/// Uses `Arc<crate::mcp::Error>` so the result is `Clone` (required by `Shared`).
/// `Ok(None)` means the agent should be skipped (e.g. missing MCP auth).
pub type McpHandle = futures::future::Shared<
    tokio::sync::oneshot::Receiver<
        Result<Option<Arc<Vec<Arc<crate::mcp::Connection>>>>, Arc<crate::mcp::Error>>
    >
>;

pub struct Client<CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, RETRG, RETRF, RETRM, CUSG> {
    /// MCP Client
    pub mcp_client: Arc<crate::mcp::Client>,
    /// Default MCP authorization headers (used when ctx doesn't provide them).
    pub mcp_authorization: Option<Arc<std::collections::HashMap<String, String>>>,
    /// Retrieve router for resolving remote agent references.
    pub retrieve_router: Arc<crate::retrieval::retrieve::Router<RETRG, RETRF, RETRM, CTXEXT>>,
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
    _marker: std::marker::PhantomData<CTXEXT>,
}

impl<CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, RETRG, RETRF, RETRM, CUSG> Client<CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, RETRG, RETRF, RETRM, CUSG> {
    pub fn new(
        mcp_client: Arc<crate::mcp::Client>,
        mcp_authorization: Option<Arc<std::collections::HashMap<String, String>>>,
        retrieve_router: Arc<crate::retrieval::retrieve::Router<RETRG, RETRF, RETRM, CTXEXT>>,
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
            mcp_authorization,
            retrieve_router,
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
            _marker: std::marker::PhantomData,
        }
    }
}

impl<CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, RETRG, RETRF, RETRM, CUSG> Clone
    for Client<CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, RETRG, RETRF, RETRM, CUSG>
{
    fn clone(&self) -> Self {
        Self {
            mcp_client: self.mcp_client.clone(),
            mcp_authorization: self.mcp_authorization.clone(),
            retrieve_router: self.retrieve_router.clone(),
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
            _marker: std::marker::PhantomData,
        }
    }
}

impl<CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, RETRG, RETRF, RETRM, CUSG> Client<CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, RETRG, RETRF, RETRM, CUSG>
where
    CTXEXT: ctx::ContextExt + Send + Sync + 'static,
    OPENROUTER: super::UpstreamClient<objectiveai::agent::openrouter::Agent> + Send + Sync + 'static,
    CLAUDEAGENTSDK: super::UpstreamClient<objectiveai::agent::claude_agent_sdk::Agent> + Send + Sync + 'static,
    MOCK: super::UpstreamClient<objectiveai::agent::mock::Agent> + Send + Sync + 'static,
    RETRG: crate::retrieval::retrieve::Client<CTXEXT>,
    RETRF: crate::retrieval::retrieve::Client<CTXEXT>,
    RETRM: crate::retrieval::retrieve::Client<CTXEXT>,
    CUSG: super::usage_handler::UsageHandler<CTXEXT> + Send + Sync + 'static,
{
    /// Creates a unary agent completion, tracking usage after completion.
    ///
    /// Internally streams the response and aggregates chunks into a single response.
    pub async fn create_unary_handle_usage(
        self: Arc<Self>,
        ctx: ctx::Context<CTXEXT, impl crate::ctx::persistent_cache::PersistentCacheClient>,
        params: Arc<objectiveai::agent::completions::request::AgentCompletionCreateParams>,
        continuation: Option<
            super::Continuation<
                OPENROUTER::State,
                CLAUDEAGENTSDK::State,
                MOCK::State,
            >,
        >,
        invention_tools: Option<Vec<objectiveai::functions::inventions::InventionTool>>,
        invention_done: Option<Arc<dyn Fn() -> bool + Send + Sync>>,
        transform_messages: Option<Arc<TransformMessages>>,
    ) -> Result<
        objectiveai::agent::completions::response::unary::AgentCompletion,
        super::Error,
    > {
        let mut aggregate: Option<
            objectiveai::agent::completions::response::streaming::AgentCompletionChunk,
        > = None;
        let mut stream = self
            .create_streaming_handle_usage(ctx, params, continuation, invention_tools, invention_done, transform_messages)
            .await?;
        while let Some(item) = stream.next().await {
            match item {
                super::StreamItem::Chunk(chunk) => match &mut aggregate {
                    Some(agg) => agg.push(&chunk),
                    None => aggregate = Some(chunk),
                },
                super::StreamItem::State(_) => {}
            }
        }
        Ok(aggregate.unwrap().into())
    }

    /// Creates a streaming agent completion, tracking usage after the stream ends.
    pub async fn create_streaming_handle_usage(
        self: Arc<Self>,
        ctx: ctx::Context<CTXEXT, impl crate::ctx::persistent_cache::PersistentCacheClient>,
        params: Arc<objectiveai::agent::completions::request::AgentCompletionCreateParams>,
        continuation: Option<
            super::Continuation<
                OPENROUTER::State,
                CLAUDEAGENTSDK::State,
                MOCK::State,
            >,
        >,
        invention_tools: Option<Vec<objectiveai::functions::inventions::InventionTool>>,
        invention_done: Option<Arc<dyn Fn() -> bool + Send + Sync>>,
        transform_messages: Option<Arc<TransformMessages>>,
    ) -> Result<
        impl futures::Stream<
            Item = super::StreamItem<
                super::Continuation<
                    OPENROUTER::State,
                    CLAUDEAGENTSDK::State,
                    MOCK::State,
                >,
            >,
        > + Send
        + Unpin
        + 'static,
        super::Error,
    > {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let _ = tokio::spawn(async move {
            let stream = match self
                .create_streaming(ctx.clone(), params.clone(), continuation, invention_tools, invention_done, transform_messages)
                .await
            {
                Ok(stream) => stream,
                Err(e) => {
                    let _ = tx.send(Err(e));
                    return;
                }
            };
            let mut aggregate: Option<
                objectiveai::agent::completions::response::streaming::AgentCompletionChunk,
            > = None;
            futures::pin_mut!(stream);
            while let Some(item) = stream.next().await {
                match &item {
                    super::StreamItem::Chunk(chunk) => {
                        match &mut aggregate {
                            Some(agg) => agg.push(chunk),
                            None => aggregate = Some(chunk.clone()),
                        }
                    }
                    super::StreamItem::State(_) => {}
                }
                let _ = tx.send(Ok(item));
            }
            drop(stream);
            drop(tx);
            let response: objectiveai::agent::completions::response::unary::AgentCompletion =
                aggregate.unwrap().into();
            if response.usage.any_usage() {
                self.usage_handler
                    .handle_usage(ctx, params, response)
                    .await;
            }
        });
        let mut stream =
            tokio_stream::wrappers::UnboundedReceiverStream::new(rx);
        match stream.next().await {
            Some(Ok(first)) => Ok(
                futures::stream::iter(std::iter::once(first))
                    .chain(stream.map(Result::unwrap)),
            ),
            Some(Err(e)) => Err(e),
            None => unreachable!(),
        }
    }

    pub async fn create_streaming(
        &self,
        ctx: ctx::Context<CTXEXT, impl crate::ctx::persistent_cache::PersistentCacheClient>,
        params: Arc<objectiveai::agent::completions::request::AgentCompletionCreateParams>,
        continuation: Option<
            super::Continuation<
                OPENROUTER::State,
                CLAUDEAGENTSDK::State,
                MOCK::State,
            >,
        >,
        invention_tools: Option<Vec<objectiveai::functions::inventions::InventionTool>>,
        invention_done: Option<Arc<dyn Fn() -> bool + Send + Sync>>,
        transform_messages: Option<Arc<TransformMessages>>,
    ) -> Result<
        impl futures::Stream<
            Item = super::StreamItem<
                super::Continuation<
                    OPENROUTER::State,
                    CLAUDEAGENTSDK::State,
                    MOCK::State,
                >,
            >,
        > + Send,
        super::Error,
    > {

        let created = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let id = response_id(created);

        // 1. Resolve agent + spawn MCP connections (skip if continuation).
        // 2. Extract continuation items.
        let (mut cont_items_or, mut cont_items_cas, mut cont_items_mock, attempts) = match continuation {
            Some(super::Continuation::Openrouter { items, agent, mcp_connections }) => {
                let attempts = vec![AgentAttempt {
                    agent: objectiveai::agent::InlineAgent::Openrouter(agent),
                    mcp: AgentMcp::Ready(mcp_connections),
                }];
                (items, vec![], vec![], attempts)
            }
            Some(super::Continuation::ClaudeAgentSdk { items, agent, mcp_connections }) => {
                let attempts = vec![AgentAttempt {
                    agent: objectiveai::agent::InlineAgent::ClaudeAgentSdk(agent),
                    mcp: AgentMcp::Ready(mcp_connections),
                }];
                (vec![], items, vec![], attempts)
            }
            Some(super::Continuation::Mock { items, agent, mcp_connections }) => {
                let attempts = vec![AgentAttempt {
                    agent: objectiveai::agent::InlineAgent::Mock(agent),
                    mcp: AgentMcp::Ready(mcp_connections),
                }];
                (vec![], vec![], items, attempts)
            }
            None => {
                let agent_wf = self.retrieve_router.get_agent(&ctx, params.agent.clone()).await
                    .map_err(|e| super::Error::InvalidAgent(e.message.to_string()))?;
                let mcp_handles = self.resolve_agents_mcp_connections(
                    &agent_wf,
                    &ctx,
                );
                let inline = agent_wf.inline();
                let mut agents: Vec<objectiveai::agent::InlineAgent> = vec![inline.inner.clone()];
                if let Some(fallbacks) = &inline.fallbacks {
                    agents.extend(fallbacks.iter().cloned());
                }
                let attempts = agents.into_iter().zip(mcp_handles).map(|(agent, handle)| {
                    AgentAttempt { agent, mcp: AgentMcp::Handle(handle) }
                }).collect();
                (vec![], vec![], vec![], attempts)
            }
        };

        // 3. Build the list of (InlineAgent, MCP connections) to try.
        //
        // For continuations, we have a single agent with existing connections.
        // For fresh requests, we have primary + fallbacks with spawned MCP handles.
        struct AgentAttempt {
            agent: objectiveai::agent::InlineAgent,
            mcp: AgentMcp,
        }
        enum AgentMcp {
            /// Already connected (from continuation).
            Ready(Arc<Vec<Arc<crate::mcp::Connection>>>),
            /// Spawned, await lazily.
            Handle(McpHandle),
        }

        // 3. Backoff retry loop — try each agent in order.
        let mut backoff = backoff::ExponentialBackoff {
            current_interval: self.backoff_current_interval,
            initial_interval: self.backoff_initial_interval,
            randomization_factor: self.backoff_randomization_factor,
            multiplier: self.backoff_multiplier,
            max_interval: self.backoff_max_interval,
            start_time: std::time::Instant::now(),
            max_elapsed_time: Some(self.backoff_max_elapsed_time),
            clock: backoff::SystemClock::default(),
        };

        loop {
            let mut errors: Vec<super::Error> = Vec::new();

            for attempt in &attempts {
                // Await MCP connections for THIS agent only.
                let mcp_connections: Arc<Vec<Arc<crate::mcp::Connection>>> = match &attempt.mcp {
                    AgentMcp::Ready(conns) => conns.clone(),
                    AgentMcp::Handle(handle) => {
                        match handle.clone().await.expect("MCP connection task panicked") {
                            Ok(Some(conns)) => conns,
                            Ok(None) => continue, // skip — missing MCP auth
                            Err(mcp_err) => {
                                errors.push(super::Error::McpConnectionArc(mcp_err));
                                continue;
                            }
                        }
                    }
                };

                // a. List MCP tools for each connection.
                let mut mcp_tools = Vec::new();
                let mut mcp_ok = true;
                for conn in mcp_connections.iter() {
                    match conn.list_tools().await {
                        Ok(tools) => mcp_tools.push(tools),
                        Err(e) => {
                            errors.push(super::Error::McpListTools {
                                url: conn.url.clone(),
                                error: e,
                            });
                            mcp_ok = false;
                            break;
                        }
                    }
                }
                if !mcp_ok {
                    continue;
                }

                // b. Resolve response format for this agent.
                let response_format = resolve_response_format(attempt.agent.id(), &params);

                // c. Resolve tools.
                let (tool_names, tool_map) = super::tool::resolve_tools(
                    &mcp_connections,
                    &mcp_tools,
                    invention_tools.as_deref(),
                    response_format.as_ref(),
                );

                // d. Get BYOK for this agent's upstream.
                let byok = ctx.upstream_authorization(attempt.agent.base().upstream()).await;

                // e. BYOK strategy: try with key first, then without.
                let byok_attempts: Vec<Option<&str>> = match &byok {
                    Some(key) => vec![Some(key.as_str()), None],
                    None => vec![None],
                };

                let agent_transform = transform_messages.as_ref().and_then(|tm| {
                    tm.get(attempt.agent.id()).map(|f| f.as_ref())
                });

                for byok_attempt in &byok_attempts {
                    let err = match &attempt.agent {
                        objectiveai::agent::InlineAgent::Openrouter(or_agent) => {
                            let a = or_agent.clone();
                            let c = mcp_connections.clone();
                            match self.run_agent_loop(
                                self.openrouter.clone(), or_agent, &params, &mcp_connections,
                                invention_tools.as_deref(), &tool_names, &tool_map,
                                &mut cont_items_or, &id, created,
                                *byok_attempt, ctx.cost_multiplier,
                                move |items| super::Continuation::Openrouter {
                                    items, agent: a, mcp_connections: c,
                                },
                                |e| super::Error::UpstreamOpenrouter(Box::new(e)),
                                objectiveai::agent::InlineAgentRef::Openrouter(&or_agent.base),
                                invention_done.clone(),
                                agent_transform,
                            ).await {
                                Ok(stream) => return Ok(stream),
                                Err(e) => e,
                            }
                        }
                        objectiveai::agent::InlineAgent::ClaudeAgentSdk(cas_agent) => {
                            let a = cas_agent.clone();
                            let c = mcp_connections.clone();
                            match self.run_agent_loop(
                                self.claude_agent_sdk.clone(), cas_agent, &params, &mcp_connections,
                                invention_tools.as_deref(), &tool_names, &tool_map,
                                &mut cont_items_cas, &id, created,
                                *byok_attempt, ctx.cost_multiplier,
                                move |items| super::Continuation::ClaudeAgentSdk {
                                    items, agent: a, mcp_connections: c,
                                },
                                |e| super::Error::UpstreamClaudeAgentSdk(Box::new(e)),
                                objectiveai::agent::InlineAgentRef::ClaudeAgentSdk(&cas_agent.base),
                                invention_done.clone(),
                                agent_transform,
                            ).await {
                                Ok(stream) => return Ok(stream),
                                Err(e) => e,
                            }
                        }
                        objectiveai::agent::InlineAgent::Mock(mock_agent) => {
                            let a = mock_agent.clone();
                            let c = mcp_connections.clone();
                            match self.run_agent_loop(
                                self.mock.clone(), mock_agent, &params, &mcp_connections,
                                invention_tools.as_deref(), &tool_names, &tool_map,
                                &mut cont_items_mock, &id, created,
                                *byok_attempt, ctx.cost_multiplier,
                                move |items| super::Continuation::Mock {
                                    items, agent: a, mcp_connections: c,
                                },
                                |e| super::Error::UpstreamMock(Box::new(e)),
                                objectiveai::agent::InlineAgentRef::Mock(&mock_agent.base),
                                invention_done.clone(),
                                agent_transform,
                            ).await {
                                Ok(stream) => return Ok(stream),
                                Err(e) => e,
                            }
                        }
                    };
                    errors.push(err);
                }
            }

            // All agents failed this round — apply backoff or give up.
            if errors.is_empty() {
                return Err(super::Error::NoAgentsResolved);
            }
            use backoff::backoff::Backoff;
            match backoff.next_backoff() {
                Some(d) => tokio::time::sleep(d).await,
                None => {
                    return Err(if errors.len() == 1 {
                        errors.into_iter().next().unwrap()
                    } else {
                        super::Error::MultipleErrors(errors)
                    });
                }
            }
        }
    }

    /// Creates an upstream stream and runs the tool-calling loop.
    ///
    /// 1. Calls `upstream.create()` with `first_chunk_timeout`.
    /// 2. Returns a stream that yields chunks as they arrive, executes
    ///    callable tools (MCP and invention), and re-invokes the upstream
    ///    for each continuation until no more callable tool calls remain.
    /// 3. The final stream item is always `StreamItem::State(CONT)`.
    ///
    /// On success, takes ownership of `cont_items` (via `std::mem::take`).
    /// On failure, `cont_items` remains intact for BYOK retry.
    async fn run_agent_loop<A, U, CONT>(
        &self,
        upstream: Arc<U>,
        agent: &A,
        params: &objectiveai::agent::completions::request::AgentCompletionCreateParams,
        mcp_connections: &[Arc<crate::mcp::Connection>],
        invention_tools: Option<&[objectiveai::functions::inventions::InventionTool]>,
        tool_names: &[String],
        tool_map: &HashMap<String, super::tool::ResolvedTool>,
        cont_items: &mut Vec<super::ContinuationItem<U::State>>,
        id: &str,
        created: u64,
        byok: Option<&str>,
        cost_multiplier: rust_decimal::Decimal,
        wrap_continuation: impl FnOnce(Vec<super::ContinuationItem<U::State>>) -> CONT + Send + 'static,
        map_upstream_err: impl Fn(U::Error) -> super::Error + Send + 'static,
        agent_base: objectiveai::agent::InlineAgentRef<'_>,
        invention_done: Option<Arc<dyn Fn() -> bool + Send + Sync>>,
        transform_messages: Option<&(dyn Fn(Vec<objectiveai::agent::completions::message::Message>) -> Vec<objectiveai::agent::completions::message::Message> + Send + Sync)>,
    ) -> Result<
        Pin<Box<dyn futures::Stream<Item = super::StreamItem<CONT>> + Send>>,
        super::Error,
    >
    where
        U: super::UpstreamClient<A> + Send + Sync + 'static,
        A: Send + Sync + Clone + 'static,
        CONT: Send + 'static,
    {
        // --- Merge messages, prepare, and apply transform. ---
        let mut messages = agent_base.merged_messages(params.messages.clone());
        objectiveai::agent::completions::message::prompt::prepare(&mut messages);
        let messages = match transform_messages {
            Some(f) => f(messages),
            None => messages,
        };

        // --- Create the initial upstream stream with timeout. ---
        let cont_ref = if cont_items.is_empty() {
            None
        } else {
            Some(cont_items.as_slice())
        };
        let create_fut = upstream.create(
            id,
            created,
            agent,
            params,
            &messages,
            mcp_connections,
            invention_tools,
            tool_names,
            tool_map,
            cont_ref,
            byok,
            cost_multiplier,
            true,
        );
        let initial_stream =
            tokio::time::timeout(self.first_chunk_timeout, create_fut)
                .await
                .map_err(|_| super::Error::Timeout)?
                .map_err(&map_upstream_err)?;

        // Success — take ownership of continuation items and build the stream.
        let mut continuation_items = std::mem::take(cont_items);
        let other_chunk_timeout = self.other_chunk_timeout;
        let agent = agent.clone();
        let mcp_connections = mcp_connections.to_vec();
        let params = params.clone();
        let invention_tools = invention_tools.map(|s| s.to_vec());
        let tool_names = tool_names.to_vec();
        let tool_map = tool_map.clone();
        let id = id.to_string();
        let byok = byok.map(|s| s.to_string());

        Ok(Box::pin(async_stream::stream! {
            use objectiveai::agent::completions::message::{RichContent, ToolMessage};

            let mut aggregate: Option<
                objectiveai::agent::completions::response::streaming::AgentCompletionChunk,
            > = None;
            let mut usage =
                objectiveai::agent::completions::response::Usage::default();
            let mut upstream_kind = objectiveai::agent::Upstream::Unknown;
            let mut final_error: Option<objectiveai::error::ResponseError> = None;
            let mut stream: Pin<Box<dyn futures::Stream<Item = super::StreamItem<U::State>> + Send>> =
                Box::pin(initial_stream);
            loop {
                let mut current_state: Option<U::State> = None;
                let mut had_error = false;
                let mut pending_chunk: Option<
                    objectiveai::agent::completions::response::streaming::AgentCompletionChunk,
                > = None;

                loop {
                    match tokio::time::timeout(other_chunk_timeout, stream.next()).await {
                        Ok(Some(super::StreamItem::Chunk(chunk))) => {
                            // Import usage from assistant response chunks.
                            for msg in &chunk.messages {
                                if let objectiveai::agent::completions::response::streaming::MessageChunk::Assistant(asst) = msg {
                                    if let Some(upstream_usage) = &asst.usage {
                                        usage.push_upstream_usage(upstream_usage);
                                    }
                                }
                            }
                            // Track upstream from the first chunk that sets it.
                            if upstream_kind == objectiveai::agent::Upstream::Unknown
                                && chunk.upstream != objectiveai::agent::Upstream::Unknown
                            {
                                upstream_kind = chunk.upstream;
                            }
                            // An error chunk means the upstream failed mid-stream.
                            // Keep draining but prevent further continuation.
                            if chunk.error.is_some() {
                                had_error = true;
                            }
                            match &mut aggregate {
                                Some(agg) => agg.push(&chunk),
                                None => aggregate = Some(chunk.clone()),
                            }
                            // Yield the previous pending chunk (without usage),
                            // buffer the current one.
                            if let Some(prev) = pending_chunk.replace(chunk) {
                                yield super::StreamItem::Chunk(prev);
                            }
                        }
                        Ok(Some(super::StreamItem::State(state))) => {
                            current_state = Some(state);
                            break;
                        }
                        Ok(None) => break,
                        Err(_) => {
                            had_error = true;
                            break;
                        }
                    }
                }

                // Yield the last buffered chunk.
                if let Some(last) = pending_chunk.take() {
                    yield super::StreamItem::Chunk(last);
                }

                if had_error {
                    break;
                }

                let Some(ref agg) = aggregate else { break };

                let callable = extract_callable_tool_calls(agg, &tool_map);

                if callable.is_empty() {
                    break;
                }

                if let Some(state) = current_state.take() {
                    continuation_items.push(super::ContinuationItem::State(state));
                }

                let mut any_invention_tool_called = false;
                for (call_id, call_name, call_args) in &callable {
                    match tool_map.get(call_name) {
                        Some(super::tool::ResolvedTool::Mcp { connection, tool }) => {
                            let args: Option<indexmap::IndexMap<String, serde_json::Value>> =
                                serde_json::from_str(call_args).ok();
                            match connection
                                .call_tool_as_message(
                                    &crate::mcp::tool::CallToolRequestParams {
                                        name: tool.name.clone(),
                                        arguments: args,
                                        _meta: None,
                                        task: None,
                                    },
                                    call_id.clone(),
                                )
                                .await
                            {
                                Ok(tool_msg) => {
                                    let idx = continuation_items.len() as u64;
                                    let chunk = make_tool_chunk(&id, created, upstream_kind, idx, &tool_msg);
                                    if let Some(ref mut agg) = aggregate {
                                        agg.push(&chunk);
                                    }
                                    yield super::StreamItem::Chunk(chunk);
                                    continuation_items
                                        .push(super::ContinuationItem::ToolMessage(tool_msg));
                                }
                                Err(_) => {
                                    had_error = true;
                                    break;
                                }
                            }
                        }
                        Some(super::tool::ResolvedTool::InventionTool(inv)) => {
                            any_invention_tool_called = true;
                            let args: serde_json::Value = serde_json::from_str(call_args)
                                .unwrap_or(serde_json::Value::Object(Default::default()));
                            let content = match (inv.call)(args).await {
                                Ok(text) => text,
                                Err(text) => format!("Error: {text}"),
                            };
                            let tool_msg = ToolMessage {
                                content: RichContent::Text(content),
                                tool_call_id: call_id.clone(),
                            };
                            let idx = continuation_items.len() as u64;
                            let chunk = make_tool_chunk(&id, created, upstream_kind, idx, &tool_msg);
                            if let Some(ref mut agg) = aggregate {
                                agg.push(&chunk);
                            }
                            yield super::StreamItem::Chunk(chunk);
                            continuation_items
                                .push(super::ContinuationItem::ToolMessage(tool_msg));
                        }
                        _ => {}
                    }
                }

                if had_error {
                    break;
                }

                // When invention_done signals completion, disable tools so the
                // model responds with content and the loop terminates naturally.
                let tools_enabled = if any_invention_tool_called {
                    !invention_done.as_ref().is_some_and(|f| f())
                } else {
                    true
                };

                // Reset aggregate so the next iteration doesn't carry
                // old tool calls forward from the previous response.
                aggregate = None;

                match upstream
                    .create(
                        &id,
                        created,
                        &agent,
                        &params,
                        &messages,
                        &mcp_connections,
                        invention_tools.as_deref(),
                        &tool_names,
                        &tool_map,
                        Some(&continuation_items),
                        byok.as_deref(),
                        cost_multiplier,
                        tools_enabled,
                    )
                    .await
                {
                    Ok(new_stream) => {
                        stream = Box::pin(new_stream);
                    }
                    Err(e) => {
                        use objectiveai::error::StatusError;
                        let e = map_upstream_err(e);
                        final_error = Some(objectiveai::error::ResponseError {
                            code: e.status(),
                            message: e.message()
                                .unwrap_or(serde_json::Value::Null),
                        });
                        break;
                    }
                }
            }

            // Single site for usage (and error if a continuation call failed).
            yield super::StreamItem::Chunk(
                objectiveai::agent::completions::response::streaming::AgentCompletionChunk {
                    id: id.clone(),
                    created,
                    upstream: upstream_kind,
                    usage: Some(usage),
                    error: final_error,
                    ..Default::default()
                },
            );
            let cont = wrap_continuation(continuation_items);
            yield super::StreamItem::State(cont);
        }))
    }

    /// Resolves agents and connects to their MCP servers concurrently.
    ///
    /// If `continuation` is provided, returns the agent and MCP connections
    /// stored in it directly (single-element vec, no spawned tasks).
    ///
    /// Otherwise, for each agent in `params` (primary + fallbacks), spawns a
    /// Spawns MCP connections for all agents (primary + fallbacks) simultaneously.
    ///
    /// This is non-async — it spawns tasks immediately and returns handles.
    /// No network waiting happens here. The backoff/retry loop awaits each
    /// handle lazily as it tries each agent.
    ///
    pub fn resolve_agents_mcp_connections(
        &self,
        agent: &objectiveai::agent::AgentWithFallbacks,
        ctx: &crate::ctx::Context<CTXEXT, impl crate::ctx::persistent_cache::PersistentCacheClient>,
    ) -> Vec<McpHandle> {
        let inline = agent.inline();
        let mut handles = Vec::with_capacity(
            1 + inline.fallbacks.as_ref().map_or(0, |f| f.len()),
        );

        // Primary
        handles.push(self.spawn_agent_mcp_connections(&inline.inner, ctx));

        // Fallbacks
        if let Some(fallbacks) = &inline.fallbacks {
            for fallback in fallbacks {
                handles.push(self.spawn_agent_mcp_connections(fallback, ctx));
            }
        }

        handles
    }

    /// Resolves a request agent (inline or remote reference) into a validated Agent
    /// and connects to its MCP servers.
    ///
    /// Returns `Ok(None)` if:
    /// - The agent's upstream kind doesn't match the continuation
    /// - An MCP server requires authorization but none was provided
    /// Spawns MCP server connections for a single agent.
    ///
    /// Returns `None` synchronously if an MCP server requires authorization
    /// but none was provided (agent should be skipped).
    /// Returns `Some(Shared<...>)` that can be awaited multiple times (for
    /// backoff retries). The task is spawned immediately — no network
    /// waiting happens here.
    pub fn spawn_agent_mcp_connections(
        &self,
        agent: &objectiveai::agent::InlineAgent,
        ctx: &crate::ctx::Context<CTXEXT, impl crate::ctx::persistent_cache::PersistentCacheClient>,
    ) -> McpHandle {
        match agent.base().mcp_servers() {
            Some(servers) if !servers.is_empty() => {
                let server_urls: Vec<_> = servers.iter().map(|s| (s.url.clone(), s.authorization)).collect();
                let (tx, rx) = tokio::sync::oneshot::channel();
                let mcp_client = self.mcp_client.clone();
                let self_mcp_auth = self.mcp_authorization.clone();
                let ctx = ctx.clone();
                tokio::spawn(async move {
                    let mcp_auth = ctx.mcp_authorization().await;
                    let mut connect_args = Vec::with_capacity(server_urls.len());
                    for (url, requires_auth) in &server_urls {
                        let authorization = if *requires_auth {
                            match mcp_auth.as_ref().and_then(|m| m.get(url))
                                .or_else(|| self_mcp_auth.as_ref().and_then(|m| m.get(url)))
                            {
                                Some(auth) => Some(auth.clone()),
                                None => {
                                    let _ = tx.send(Ok(None)); // skip agent
                                    return;
                                }
                            }
                        } else {
                            None
                        };
                        connect_args.push((url.clone(), authorization));
                    }
                    let mut futs = Vec::with_capacity(connect_args.len());
                    for (url, auth) in connect_args {
                        futs.push(mcp_client.connect(url, auth));
                    }
                    let results = futures::future::join_all(futs).await;
                    let mut connections = Vec::with_capacity(results.len());
                    for result in results {
                        match result {
                            Ok(conn) => connections.push(conn),
                            Err(e) => {
                                let _ = tx.send(Err(Arc::new(e)));
                                return;
                            }
                        }
                    }
                    let _ = tx.send(Ok(Some(Arc::new(connections))));
                });
                futures::FutureExt::shared(rx)
            }
            _ => {
                let (tx, rx) = tokio::sync::oneshot::channel();
                let _ = tx.send(Ok(Some(Arc::new(Vec::new()))));
                futures::FutureExt::shared(rx)
            }
        }
    }
}

/// Resolves the response format for a given agent from the request params.
fn resolve_response_format(
    agent_id: &str,
    params: &objectiveai::agent::completions::request::AgentCompletionCreateParams,
) -> Option<objectiveai::agent::completions::request::ResponseFormat> {
    use objectiveai::agent::completions::request::ResponseFormatParam;
    match params.response_format.as_ref()? {
        ResponseFormatParam::Single(rf) => Some(rf.clone()),
        ResponseFormatParam::PerAgent(map) => map.get(agent_id).cloned(),
    }
}

/// Extracts callable tool calls (MCP and invention) from the accumulated chunk.
/// Returns `(call_id, resolved_tool_name, arguments_json)` for each callable tool.
fn extract_callable_tool_calls(
    aggregate: &objectiveai::agent::completions::response::streaming::AgentCompletionChunk,
    tool_map: &HashMap<String, super::tool::ResolvedTool>,
) -> Vec<(String, String, String)> {
    use objectiveai::agent::completions::response::streaming::MessageChunk;

    let mut callable = Vec::new();
    // Find the last assistant message and extract its accumulated tool calls.
    for msg in aggregate.messages.iter().rev() {
        if let MessageChunk::Assistant(chunk) = msg {
            if let Some(tool_calls) = &chunk.tool_calls {
                for tc in tool_calls {
                    let id = tc.id.clone().unwrap_or_default();
                    let name = tc
                        .function
                        .as_ref()
                        .and_then(|f| f.name.clone())
                        .unwrap_or_default();
                    let args = tc
                        .function
                        .as_ref()
                        .and_then(|f| f.arguments.clone())
                        .unwrap_or_default();
                    if name.is_empty() {
                        continue;
                    }
                    match tool_map.get(&name) {
                        Some(super::tool::ResolvedTool::Mcp { .. })
                        | Some(super::tool::ResolvedTool::InventionTool(_)) => {
                            callable.push((id, name, args));
                        }
                        _ => {}
                    }
                }
            }
            break; // only inspect the last assistant message
        }
    }
    callable
}

/// Builds an `AgentCompletionChunk` containing a single tool-response message.
fn make_tool_chunk(
    id: &str,
    created: u64,
    upstream: objectiveai::agent::Upstream,
    index: u64,
    tool_msg: &objectiveai::agent::completions::message::ToolMessage,
) -> objectiveai::agent::completions::response::streaming::AgentCompletionChunk {
    use objectiveai::agent::completions::response::streaming::{
        AgentCompletionChunk, MessageChunk,
    };
    use objectiveai::agent::completions::response::ToolResponse;
    AgentCompletionChunk {
        id: id.to_string(),
        created,
        upstream,
        messages: vec![MessageChunk::Tool(ToolResponse {
            role: Default::default(),
            index,
            inner: tool_msg.clone(),
        })],
        ..Default::default()
    }
}
