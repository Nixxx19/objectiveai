use std::{collections::HashMap, pin::Pin, sync::Arc, time::Duration};

use crate::ctx;
use futures::StreamExt;

pub fn response_id(created: u64) -> String {
    let uuid = uuid::Uuid::new_v4();
    format!("agtcpl-{}-{created}", uuid.simple())
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

/// Runs the streaming + tool-calling loop for a locked agent.
///
/// Streams chunks to `tx`, executes callable tools (MCP and invention),
/// re-invokes the upstream for each continuation until no more callable
/// tool calls remain, then sends the final `Continuation` as the last
/// stream item.
///
/// Returns the accumulated `AgentCompletionChunk` for usage tracking,
/// or `None` if no chunks were produced.
async fn run_agent_loop<A, U, CONT>(
    tx: &tokio::sync::mpsc::UnboundedSender<super::StreamItem<CONT>>,
    upstream: &U,
    agent: &A,
    mcp_connections: &[Arc<crate::mcp::Connection>],
    params: &objectiveai::agent::completions::request::AgentCompletionCreateParams,
    invention_tools: Option<&[objectiveai::functions::inventions::InventionTool]>,
    tool_names: &[String],
    tool_map: &HashMap<String, super::tool::ResolvedTool>,
    initial_stream: U::Stream,
    mut continuation_items: Vec<super::ContinuationItem<U::State>>,
    id: &str,
    created: u64,
    other_chunk_timeout: Duration,
    byok: Option<&str>,
    cost_multiplier: rust_decimal::Decimal,
    wrap_continuation: impl FnOnce(Vec<super::ContinuationItem<U::State>>) -> CONT,
) -> Option<objectiveai::agent::completions::response::streaming::AgentCompletionChunk>
where
    U: super::UpstreamClient<A> + Send + Sync,
    U::State: Send + Sync,
    U::Stream: Send + 'static,
    A: Send + Sync,
    CONT: Send + 'static,
{
    use objectiveai::agent::completions::message::{RichContent, ToolMessage};

    let mut aggregate: Option<
        objectiveai::agent::completions::response::streaming::AgentCompletionChunk,
    > = None;
    let mut stream: Pin<Box<dyn futures::Stream<Item = super::StreamItem<U::State>> + Send>> =
        Box::pin(initial_stream);

    loop {
        // --- Stream all items from the current upstream stream. ---
        let mut current_state: Option<U::State> = None;
        let mut had_error = false;

        loop {
            match tokio::time::timeout(other_chunk_timeout, stream.next()).await {
                Ok(Some(super::StreamItem::Chunk(chunk))) => {
                    match &mut aggregate {
                        Some(agg) => agg.push(&chunk),
                        None => aggregate = Some(chunk.clone()),
                    }
                    if tx.send(super::StreamItem::Chunk(chunk)).is_err() {
                        return aggregate; // receiver dropped
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

        if had_error {
            break;
        }

        let Some(ref agg) = aggregate else { break };

        // --- Check for callable tool calls in the accumulated response. ---
        let callable = extract_callable_tool_calls(agg, tool_map);

        if callable.is_empty() {
            // Done — send the final continuation.
            let cont = wrap_continuation(continuation_items);
            let _ = tx.send(super::StreamItem::State(cont));
            break;
        }

        // --- Save the current upstream state as a continuation item. ---
        if let Some(state) = current_state.take() {
            continuation_items.push(super::ContinuationItem::State(state));
        }

        // --- Execute each callable tool. ---
        let next_index_base = agg.messages.len() as u64;
        let mut next_idx = next_index_base;

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
                            let chunk = make_tool_chunk(id, created, next_idx, &tool_msg);
                            if let Some(ref mut agg) = aggregate {
                                agg.push(&chunk);
                            }
                            let _ = tx.send(super::StreamItem::Chunk(chunk));
                            continuation_items
                                .push(super::ContinuationItem::ToolMessage(tool_msg));
                            next_idx += 1;
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
                    let chunk = make_tool_chunk(id, created, next_idx, &tool_msg);
                    if let Some(ref mut agg) = aggregate {
                        agg.push(&chunk);
                    }
                    let _ = tx.send(super::StreamItem::Chunk(chunk));
                    continuation_items.push(super::ContinuationItem::ToolMessage(tool_msg));
                    next_idx += 1;
                }
                _ => {} // ResponseFormat or unknown — not callable
            }
        }

        if had_error {
            break;
        }

        // --- Create a new upstream stream with the updated continuation. ---
        match upstream
            .create(
                id,
                created,
                agent,
                params,
                &params.messages,
                mcp_connections,
                invention_tools,
                tool_names,
                tool_map,
                Some(&continuation_items),
                byok,
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

    aggregate
}

/// Attempts to create an upstream stream and spawn the agent loop.
///
/// On success, takes ownership of `cont_items` (via `std::mem::take`) and
/// spawns a background task running `run_agent_loop`. Returns a boxed stream.
/// On failure, `cont_items` remains intact for BYOK retry.
async fn try_create_and_spawn<A, U, CTXEXT, CUSG, CONT>(
    upstream: Arc<U>,
    usage_handler: Arc<CUSG>,
    agent: &A,
    mcp_connections: &[Arc<crate::mcp::Connection>],
    params: &Arc<objectiveai::agent::completions::request::AgentCompletionCreateParams>,
    ctx: &ctx::Context<CTXEXT>,
    invention_tools: &Option<Vec<objectiveai::functions::inventions::InventionTool>>,
    tool_names: &[String],
    tool_map: &HashMap<String, super::tool::ResolvedTool>,
    cont_items: &mut Vec<super::ContinuationItem<U::State>>,
    id: &str,
    created: u64,
    first_chunk_timeout: Duration,
    other_chunk_timeout: Duration,
    byok: Option<&str>,
    wrap_continuation: impl FnOnce(Vec<super::ContinuationItem<U::State>>) -> CONT + Send + 'static,
) -> Result<
    Pin<Box<dyn futures::Stream<Item = super::StreamItem<CONT>> + Send>>,
    super::Error,
>
where
    U: super::UpstreamClient<A> + Send + Sync + 'static,
    U::State: Send + Sync + 'static,
    U::Stream: Send + 'static,
    A: Send + Sync + Clone + 'static,
    CTXEXT: Send + Sync + 'static,
    CUSG: super::usage_handler::UsageHandler<CTXEXT> + Send + Sync + 'static,
    CONT: Send + 'static,
{
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
        &params.messages,
        mcp_connections,
        invention_tools.as_deref(),
        tool_names,
        tool_map,
        cont_ref,
        byok,
        ctx.cost_multiplier,
    );
    let (stream, _state) = tokio::time::timeout(first_chunk_timeout, create_fut)
        .await
        .map_err(|_| super::Error::Timeout)?
        .map_err(super::Error::Upstream)?;

    // Success — take ownership of continuation items and spawn the loop.
    let cont_items = std::mem::take(cont_items);
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
    let agent = agent.clone();
    let mcp_connections = mcp_connections.to_vec();
    let params = params.clone();
    let ctx = ctx.clone();
    let invention_tools = invention_tools.clone();
    let tool_names = tool_names.to_vec();
    let tool_map = tool_map.clone();
    let id = id.to_string();
    let byok_owned = byok.map(|s| s.to_string());

    tokio::spawn(async move {
        let aggregate = run_agent_loop(
            &tx,
            &*upstream,
            &agent,
            &mcp_connections,
            &params,
            invention_tools.as_deref(),
            &tool_names,
            &tool_map,
            stream,
            cont_items,
            &id,
            created,
            other_chunk_timeout,
            byok_owned.as_deref(),
            ctx.cost_multiplier,
            wrap_continuation,
        )
        .await;
        drop(tx);
        if let Some(agg) = aggregate {
            usage_handler
                .handle_usage(ctx, params, agg.into())
                .await;
        }
    });

    let mut rx_stream = tokio_stream::wrappers::UnboundedReceiverStream::new(rx);
    match rx_stream.next().await {
        Some(item @ super::StreamItem::Chunk(_)) => Ok(Box::pin(
            futures::stream::iter(std::iter::once(item)).chain(rx_stream),
        )),
        _ => Err(super::Error::EmptyStream),
    }
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
        > + Send
            + 'static,
        super::Error,
    > {

        let created = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let id = response_id(created);
        let first_chunk_timeout = self.first_chunk_timeout;
        let other_chunk_timeout = self.other_chunk_timeout;

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

                for byok_attempt in &byok_attempts {
                    let result = match agent {
                        objectiveai::agent::Agent::Openrouter(or_agent) => {
                            let a = or_agent.clone();
                            let c = mcp_connections.clone();
                            try_create_and_spawn(
                                self.openrouter.clone(), self.usage_handler.clone(),
                                or_agent, mcp_connections, &params, &ctx, &invention_tools,
                                &tool_names, &tool_map, &mut cont_items_or,
                                &id, created, first_chunk_timeout, other_chunk_timeout,
                                *byok_attempt,
                                move |items| super::Continuation::Openrouter {
                                    items, agent: a, mcp_connections: c,
                                },
                            ).await
                        }
                        objectiveai::agent::Agent::ClaudeAgentSdk(cas_agent) => {
                            let a = cas_agent.clone();
                            let c = mcp_connections.clone();
                            try_create_and_spawn(
                                self.claude_agent_sdk.clone(), self.usage_handler.clone(),
                                cas_agent, mcp_connections, &params, &ctx, &invention_tools,
                                &tool_names, &tool_map, &mut cont_items_cas,
                                &id, created, first_chunk_timeout, other_chunk_timeout,
                                *byok_attempt,
                                move |items| super::Continuation::ClaudeAgentSdk {
                                    items, agent: a, mcp_connections: c,
                                },
                            ).await
                        }
                        objectiveai::agent::Agent::Mock(mock_agent) => {
                            let a = mock_agent.clone();
                            let c = mcp_connections.clone();
                            try_create_and_spawn(
                                self.mock.clone(), self.usage_handler.clone(),
                                mock_agent, mcp_connections, &params, &ctx, &invention_tools,
                                &tool_names, &tool_map, &mut cont_items_mock,
                                &id, created, first_chunk_timeout, other_chunk_timeout,
                                *byok_attempt,
                                move |items| super::Continuation::Mock {
                                    items, agent: a, mcp_connections: c,
                                },
                            ).await
                        }
                    };
                    match result {
                        Ok(stream) => return Ok(stream),
                        Err(e) => errors.push(e),
                    }
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
