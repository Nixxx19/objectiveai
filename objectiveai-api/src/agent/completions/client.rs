use std::{collections::HashMap, pin::Pin, sync::Arc, time::Duration};

use crate::ctx;
use futures::StreamExt;

/// A function that transforms messages before they are sent to an upstream.
/// Keyed by agent ID so each agent in an ensemble can receive different messages.
pub type TransformMessages = HashMap<
    String,
    Box<dyn Fn(Vec<objectiveai::agent::completions::message::Message>) -> Vec<objectiveai::agent::completions::message::Message> + Send + Sync>,
>;

pub fn response_id(created: u64) -> String {
    let uuid = uuid::Uuid::new_v4();
    format!("agtcpl-{}-{created}", uuid.simple())
}

// ---------------------------------------------------------------------------

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
    /// Creates a unary agent completion, tracking usage after completion.
    ///
    /// Internally streams the response and aggregates chunks into a single response.
    pub async fn create_unary_handle_usage(
        self: Arc<Self>,
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
        transform_messages: Option<Arc<TransformMessages>>,
    ) -> Result<
        objectiveai::agent::completions::response::unary::AgentCompletion,
        super::Error,
    > {
        let mut aggregate: Option<
            objectiveai::agent::completions::response::streaming::AgentCompletionChunk,
        > = None;
        let mut stream = self
            .create_streaming_handle_usage(ctx, params, continuation, invention_tools, transform_messages)
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
                .create_streaming(ctx.clone(), params.clone(), continuation, invention_tools, transform_messages)
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
            let mut error = false;
            futures::pin_mut!(stream);
            while let Some(item) = stream.next().await {
                match &item {
                    super::StreamItem::Chunk(chunk) => {
                        if chunk.error.is_some() {
                            error = true;
                        }
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
            if !error {
                if let Some(agg) = aggregate {
                    self.usage_handler
                        .handle_usage(ctx, params, agg.into())
                        .await;
                }
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

        // 1. Resolve agents concurrently (borrows continuation temporarily).
        let handles = self.resolve_agents(ctx.clone(), &params, continuation.as_ref());

        // 2. Extract continuation items (moves continuation).
        let (mut cont_items_or, mut cont_items_cas, mut cont_items_mock) = match continuation {
            Some(super::Continuation::Openrouter { items, .. }) => (items, vec![], vec![]),
            Some(super::Continuation::ClaudeAgentSdk { items, .. }) => (vec![], items, vec![]),
            Some(super::Continuation::Mock { items, .. }) => (vec![], vec![], items),
            None => (vec![], vec![], vec![]),
        };

        // 3. Prepare lazy resolution slots.
        //
        // Handles are awaited lazily in order: agent 1 is resolved and
        // tried before agent 2 is ever awaited. On subsequent backoff
        // iterations, already-resolved agents are retried directly.
        //
        // `None` = agent not applicable (e.g. missing MCP authorization).
        // `Failed(e)` = resolution error, preserved for final error reporting.
        enum AgentSlot {
            Pending(
                tokio::task::JoinHandle<
                    Result<
                        Option<(
                            objectiveai::agent::Agent,
                            Vec<Arc<crate::mcp::Connection>>,
                        )>,
                        super::Error,
                    >,
                >,
            ),
            Resolved(
                objectiveai::agent::Agent,
                Vec<Arc<crate::mcp::Connection>>,
            ),
            Failed(super::Error),
        }

        let mut slots: Vec<Option<AgentSlot>> =
            handles.into_iter().map(|h| Some(AgentSlot::Pending(h))).collect();

        /// Drain `Failed` errors from slots into `errors`.
        fn collect_slot_errors(
            slots: &mut [Option<AgentSlot>],
            errors: &mut Vec<super::Error>,
        ) {
            for slot in slots.iter_mut() {
                if matches!(slot, Some(AgentSlot::Failed(_))) {
                    let Some(AgentSlot::Failed(e)) = slot.take() else {
                        unreachable!()
                    };
                    errors.push(e);
                }
            }
        }

        /// Return an appropriate error from a non-empty error vec.
        fn into_error(errors: Vec<super::Error>) -> super::Error {
            if errors.len() == 1 {
                errors.into_iter().next().unwrap()
            } else {
                super::Error::MultipleErrors(errors)
            }
        }

        // 4. Backoff retry loop — try each agent in order.
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
            let mut any_resolved = false;

            for slot in &mut slots {
                let Some(inner) = slot else { continue };

                // Lazily resolve pending slots.
                if matches!(inner, AgentSlot::Pending(_)) {
                    let Some(AgentSlot::Pending(handle)) = slot.take() else {
                        unreachable!()
                    };
                    match handle.await.expect("resolve_agent task panicked") {
                        Ok(Some((agent, conns))) => {
                            *slot = Some(AgentSlot::Resolved(agent, conns));
                        }
                        Ok(None) => continue, // not applicable, slot stays None
                        Err(e) => {
                            *slot = Some(AgentSlot::Failed(e));
                            continue;
                        }
                    }
                }

                let Some(AgentSlot::Resolved(agent, mcp_connections)) = slot else {
                    continue;
                };
                any_resolved = true;

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
                let response_format = resolve_response_format(agent.id(), &params);

                // c. Resolve tools.
                let (tool_names, tool_map) = super::tool::resolve_tools(
                    mcp_connections,
                    &mcp_tools,
                    invention_tools.as_deref(),
                    response_format.as_ref(),
                );

                // d. Get BYOK for this agent's upstream.
                let byok = match ctx.ext.get_byok(agent.base().upstream()).await {
                    Ok(b) => b,
                    Err(e) => {
                        errors.push(super::Error::Upstream(e));
                        continue;
                    }
                };

                // e. BYOK strategy: try with key first, then without.
                let byok_attempts: Vec<Option<&str>> = match &byok {
                    Some(key) => vec![Some(key.as_str()), None],
                    None => vec![None],
                };

                let agent_transform = transform_messages.as_ref().and_then(|tm| {
                    tm.get(agent.id()).map(|f| f.as_ref())
                });

                for byok_attempt in &byok_attempts {
                    let err = match agent {
                        objectiveai::agent::Agent::Openrouter(or_agent) => {
                            let a = or_agent.clone();
                            let c = mcp_connections.clone();
                            match self.run_agent_loop(
                                self.openrouter.clone(), or_agent, &params, mcp_connections,
                                invention_tools.as_deref(), &tool_names, &tool_map,
                                &mut cont_items_or, &id, created,
                                *byok_attempt, ctx.cost_multiplier,
                                move |items| super::Continuation::Openrouter {
                                    items, agent: a, mcp_connections: c,
                                },
                                objectiveai::agent::AgentBaseRef::Openrouter(&or_agent.base),
                                agent_transform,
                            ).await {
                                Ok(stream) => return Ok(stream),
                                Err(e) => e,
                            }
                        }
                        objectiveai::agent::Agent::ClaudeAgentSdk(cas_agent) => {
                            let a = cas_agent.clone();
                            let c = mcp_connections.clone();
                            match self.run_agent_loop(
                                self.claude_agent_sdk.clone(), cas_agent, &params, mcp_connections,
                                invention_tools.as_deref(), &tool_names, &tool_map,
                                &mut cont_items_cas, &id, created,
                                *byok_attempt, ctx.cost_multiplier,
                                move |items| super::Continuation::ClaudeAgentSdk {
                                    items, agent: a, mcp_connections: c,
                                },
                                objectiveai::agent::AgentBaseRef::ClaudeAgentSdk(&cas_agent.base),
                                agent_transform,
                            ).await {
                                Ok(stream) => return Ok(stream),
                                Err(e) => e,
                            }
                        }
                        objectiveai::agent::Agent::Mock(mock_agent) => {
                            let a = mock_agent.clone();
                            let c = mcp_connections.clone();
                            match self.run_agent_loop(
                                self.mock.clone(), mock_agent, &params, mcp_connections,
                                invention_tools.as_deref(), &tool_names, &tool_map,
                                &mut cont_items_mock, &id, created,
                                *byok_attempt, ctx.cost_multiplier,
                                move |items| super::Continuation::Mock {
                                    items, agent: a, mcp_connections: c,
                                },
                                objectiveai::agent::AgentBaseRef::Mock(&mock_agent.base),
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
            if !any_resolved {
                collect_slot_errors(&mut slots, &mut errors);
                return Err(if errors.is_empty() {
                    super::Error::NoAgentsResolved
                } else {
                    into_error(errors)
                });
            }
            use backoff::backoff::Backoff;
            match backoff.next_backoff() {
                Some(d) => tokio::time::sleep(d).await,
                None => {
                    collect_slot_errors(&mut slots, &mut errors);
                    return Err(into_error(errors));
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
        agent_base: objectiveai::agent::AgentBaseRef<'_>,
        transform_messages: Option<&(dyn Fn(Vec<objectiveai::agent::completions::message::Message>) -> Vec<objectiveai::agent::completions::message::Message> + Send + Sync)>,
    ) -> Result<
        Pin<Box<dyn futures::Stream<Item = super::StreamItem<CONT>> + Send>>,
        super::Error,
    >
    where
        U: super::UpstreamClient<A> + Send + Sync + 'static,
        U::State: Send + Sync + 'static,
        U::Stream: Send + 'static,
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
        );
        let (initial_stream, _state) =
            tokio::time::timeout(self.first_chunk_timeout, create_fut)
                .await
                .map_err(|_| super::Error::Timeout)?
                .map_err(super::Error::Upstream)?;

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
            let mut stream: Pin<Box<dyn futures::Stream<Item = super::StreamItem<U::State>> + Send>> =
                Box::pin(initial_stream);
            loop {
                let mut current_state: Option<U::State> = None;
                let mut had_error = false;

                loop {
                    match tokio::time::timeout(other_chunk_timeout, stream.next()).await {
                        Ok(Some(super::StreamItem::Chunk(chunk))) => {
                            match &mut aggregate {
                                Some(agg) => agg.push(&chunk),
                                None => aggregate = Some(chunk.clone()),
                            }
                            yield super::StreamItem::Chunk(chunk);
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

                if had_error {
                    break;
                }

                let Some(ref agg) = aggregate else { break };

                let callable = extract_callable_tool_calls(agg, &tool_map);

                if callable.is_empty() {
                    let cont = wrap_continuation(continuation_items);
                    yield super::StreamItem::State(cont);
                    break;
                }

                if let Some(state) = current_state.take() {
                    continuation_items.push(super::ContinuationItem::State(state));
                }

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
                                    let chunk = make_tool_chunk(&id, created, idx, &tool_msg);
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
                            let chunk = make_tool_chunk(&id, created, idx, &tool_msg);
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
                    )
                    .await
                {
                    Ok((new_stream, _new_state)) => {
                        stream = Box::pin(new_stream);
                    }
                    Err(_) => break,
                }
            }
        }))
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
                match self.agent_fetcher.fetch(ctx, id).await.map_err(super::Error::Fetch)? {
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
                    connections.push(result.map_err(super::Error::McpConnection)?);
                }
                connections
            }
            _ => Vec::new(),
        };

        Ok(Some((agent, mcp_connections)))
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
        messages: vec![MessageChunk::Tool(ToolResponse {
            role: Default::default(),
            index,
            inner: tool_msg.clone(),
        })],
        ..Default::default()
    }
}

#[cfg(test)]
#[path = "client_tests.rs"]
mod tests;
