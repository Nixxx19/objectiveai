use crate::{ctx, util::ChoiceIndexer};
use futures::{Stream, StreamExt};
use std::{
    pin::Pin,
    sync::Arc,
    time,
};

type FunctionInventionChunk =
    objectiveai::functions::inventions::response::streaming::FunctionInventionChunk;
type RecursiveChunk =
    objectiveai::functions::inventions::recursive::response::streaming::FunctionInventionRecursiveChunk;
type RecursiveInventionChunk =
    objectiveai::functions::inventions::recursive::response::streaming::FunctionInventionChunk;
type RecursiveObject =
    objectiveai::functions::inventions::recursive::response::streaming::Object;

/// Generates a unique response ID for recursive Function inventions.
pub fn recursive_invention_response_id(created: u64) -> String {
    let uuid = uuid::Uuid::new_v4();
    format!("fncinvrec-{}-{}", uuid.simple(), created)
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/// Client for recursively inventing Functions.
///
/// Orchestrates the recursive invention flow: invents the root function,
/// then spawns child inventions for each placeholder task, recursing
/// based on depth. All child streams are merged concurrently — no waiting,
/// no collecting.
pub struct Client<CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, FAGENT, CUSG, IUSG, RIUSG> {
    pub invention_client: Arc<
        crate::functions::inventions::Client<
            CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, FAGENT, CUSG, IUSG,
        >,
    >,
    pub usage_handler: Arc<RIUSG>,
}

impl<CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, FAGENT, CUSG, IUSG, RIUSG>
    Client<CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, FAGENT, CUSG, IUSG, RIUSG>
{
    pub fn new(
        invention_client: Arc<
            crate::functions::inventions::Client<
                CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, FAGENT, CUSG, IUSG,
            >,
        >,
        usage_handler: Arc<RIUSG>,
    ) -> Self {
        Self {
            invention_client,
            usage_handler,
        }
    }
}

impl<CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, FAGENT, CUSG, IUSG, RIUSG>
    Client<CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, FAGENT, CUSG, IUSG, RIUSG>
where
    CTXEXT: ctx::ContextExt + Send + Sync + 'static,
    OPENROUTER: crate::agent::completions::UpstreamClient<objectiveai::agent::openrouter::Agent>
        + Send
        + Sync
        + 'static,
    CLAUDEAGENTSDK: crate::agent::completions::UpstreamClient<
            objectiveai::agent::claude_agent_sdk::Agent,
        > + Send
        + Sync
        + 'static,
    MOCK: crate::agent::completions::UpstreamClient<objectiveai::agent::mock::Agent>
        + Send
        + Sync
        + 'static,
    FAGENT: crate::agent::fetcher::Fetcher<CTXEXT> + Send + Sync + 'static,
    CUSG: crate::agent::completions::usage_handler::UsageHandler<CTXEXT> + Send + Sync + 'static,
    IUSG: crate::functions::inventions::usage_handler::UsageHandler<CTXEXT> + Send + Sync + 'static,
    RIUSG: super::usage_handler::UsageHandler<CTXEXT> + Send + Sync + 'static,
{
    pub async fn create_streaming(
        self: Arc<Self>,
        ctx: ctx::Context<CTXEXT>,
        request: Arc<objectiveai::functions::inventions::recursive::request::FunctionInventionRecursiveCreateParams>,
    ) -> Result<
        impl Stream<Item = RecursiveChunk> + Send + 'static,
        super::Error,
    > {
        let created = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let id = recursive_invention_response_id(created);

        let is_scalar = match &request.state {
            objectiveai::functions::inventions::state::ParamsState::AlphaScalarBranch(_)
            | objectiveai::functions::inventions::state::ParamsState::AlphaScalarLeaf(_)
            | objectiveai::functions::inventions::state::ParamsState::AlphaScalar(_) => true,
            _ => false,
        };
        let object = if is_scalar {
            RecursiveObject::AlphaScalarFunctionInventionRecursiveChunk
        } else {
            RecursiveObject::AlphaVectorFunctionInventionRecursiveChunk
        };

        let choice_indexer = Arc::new(ChoiceIndexer::new(0));

        let stream = run_recursive(
            self.invention_client.clone(),
            ctx,
            request,
            id.clone(),
            created,
            object,
            choice_indexer,
            0, // native index for root
        );

        Ok(stream)
    }

    pub async fn create_streaming_handle_usage(
        self: Arc<Self>,
        ctx: ctx::Context<CTXEXT>,
        request: Arc<objectiveai::functions::inventions::recursive::request::FunctionInventionRecursiveCreateParams>,
    ) -> Result<
        impl Stream<Item = RecursiveChunk> + Send + Unpin + 'static,
        super::Error,
    > {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let self_clone = self.clone();
        tokio::spawn(async move {
            let mut aggregate: Option<RecursiveChunk> = None;
            let stream = match self_clone
                .clone()
                .create_streaming(ctx.clone(), request.clone())
                .await
            {
                Ok(stream) => stream,
                Err(e) => {
                    let _ = tx.send(Err(e));
                    return;
                }
            };
            futures::pin_mut!(stream);
            while let Some(chunk) = stream.next().await {
                match &mut aggregate {
                    Some(aggregate) => aggregate.push(&chunk),
                    None => aggregate = Some(chunk.clone()),
                }
                let _ = tx.send(Ok(chunk));
            }
            drop(stream);
            drop(tx);
            if let Some(aggregate) = aggregate {
                if aggregate.usage.as_ref().is_some_and(
                    objectiveai::agent::completions::response::Usage::any_usage,
                ) {
                    self_clone
                        .usage_handler
                        .handle_usage(ctx, request, aggregate.into())
                        .await;
                }
            }
        });
        let mut stream =
            tokio_stream::wrappers::UnboundedReceiverStream::new(rx);
        match stream.next().await {
            Some(Ok(chunk)) => {
                Ok(crate::util::StreamOnce::new(chunk)
                    .chain(stream.map(Result::unwrap)))
            }
            Some(Err(e)) => Err(e),
            None => unreachable!(),
        }
    }
}

/// Recursively invents a function and all its placeholder children.
///
/// 1. Runs a single-level invention for the given state.
/// 2. Wraps each chunk with the assigned index and yields immediately.
/// 3. After the invention stream completes, extracts placeholder children
///    from the final state.
/// 4. Spawns a recursive invention for each child concurrently.
/// 5. Merges all child streams via `select_all` and yields their chunks.
/// 6. After all children complete, replaces placeholders with the invented
///    function paths.
fn run_recursive<CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, FAGENT, CUSG, IUSG>(
    invention_client: Arc<
        crate::functions::inventions::Client<
            CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, FAGENT, CUSG, IUSG,
        >,
    >,
    ctx: ctx::Context<CTXEXT>,
    request: Arc<objectiveai::functions::inventions::recursive::request::FunctionInventionRecursiveCreateParams>,
    id: String,
    created: u64,
    object: RecursiveObject,
    choice_indexer: Arc<ChoiceIndexer>,
    native_index: usize,
) -> Pin<Box<dyn Stream<Item = RecursiveChunk> + Send>>
where
    CTXEXT: ctx::ContextExt + Send + Sync + 'static,
    OPENROUTER: crate::agent::completions::UpstreamClient<objectiveai::agent::openrouter::Agent>
        + Send
        + Sync
        + 'static,
    CLAUDEAGENTSDK: crate::agent::completions::UpstreamClient<
            objectiveai::agent::claude_agent_sdk::Agent,
        > + Send
        + Sync
        + 'static,
    MOCK: crate::agent::completions::UpstreamClient<objectiveai::agent::mock::Agent>
        + Send
        + Sync
        + 'static,
    FAGENT: crate::agent::fetcher::Fetcher<CTXEXT> + Send + Sync + 'static,
    CUSG: crate::agent::completions::usage_handler::UsageHandler<CTXEXT> + Send + Sync + 'static,
    IUSG: crate::functions::inventions::usage_handler::UsageHandler<CTXEXT> + Send + Sync + 'static,
{
    Box::pin(async_stream::stream! {
        // Build the single-level invention request from the recursive params.
        let invention_request = Arc::new(
            objectiveai::functions::inventions::request::FunctionInventionCreateParams {
                remote: Some(request.remote),
                overwrite: Some(true),
                github_token: request.github_token.clone(),
                state: request.state.clone(),
                provider: request.provider.clone(),
                agent: request.agent.clone(),
                agents: request.agents.clone(),
                seed: request.seed,
                stream: request.stream,
                max_step_retries: request.max_step_retries,
                mcp_server_authorization: request.mcp_server_authorization.clone(),
            },
        );

        // Run the single-level invention.
        let stream = match invention_client
            .clone()
            .create_streaming(ctx.clone(), invention_request)
            .await
        {
            Ok(stream) => stream,
            Err(e) => {
                // Yield an error chunk and return.
                yield RecursiveChunk {
                    id: id.clone(),
                    inventions: vec![RecursiveInventionChunk {
                        index: choice_indexer.get(native_index),
                        inner: FunctionInventionChunk {
                            id: id.clone(),
                            completions: vec![],
                            state: None,
                            path: None,
                            function: None,
                            created,
                            object: object.into(),
                            usage: None,
                            error: Some(objectiveai::error::ResponseError {
                                code: objectiveai::error::StatusError::status(&e),
                                message: objectiveai::error::StatusError::message(&e)
                                    .unwrap_or_else(|| serde_json::json!(e.to_string())),
                            }),
                        },
                    }],
                    inventions_errors: Some(true),
                    created,
                    object,
                    usage: None,
                };
                return;
            }
        };

        // Stream the single-level invention, wrapping each chunk.
        let mut final_state: Option<objectiveai::functions::inventions::State> = None;
        let mut final_path: Option<objectiveai::functions::RemoteFunctionPath> = None;
        let mut had_error = false;

        futures::pin_mut!(stream);
        while let Some(chunk) = stream.next().await {
            if chunk.state.is_some() {
                final_state = chunk.state.clone();
            }
            if chunk.path.is_some() {
                final_path = chunk.path.clone();
            }
            if chunk.error.is_some() {
                had_error = true;
            }
            yield RecursiveChunk {
                id: id.clone(),
                inventions: vec![RecursiveInventionChunk {
                    index: choice_indexer.get(native_index),
                    inner: chunk,
                }],
                inventions_errors: None,
                created,
                object,
                usage: None,
            };
        }
        drop(stream);

        // If the invention errored or produced no state, stop here.
        let mut state = match final_state {
            Some(state) if !had_error => state,
            _ => return,
        };

        // Extract placeholder children from the final state.
        let children = state.placeholder_children();
        if children.is_empty() {
            return;
        }

        // Spawn a recursive invention for each child concurrently.
        // Each child gets a unique native index for the choice indexer.
        let base_native = (native_index + 1) * 1000; // avoid collisions
        let mut child_streams: Vec<Pin<Box<dyn Stream<Item = RecursiveChunk> + Send>>> = Vec::new();

        for (i, child_state) in children.into_iter().enumerate() {
            let child_native = base_native + i;

            // Build the child's recursive request with the child's state.
            let child_request = Arc::new(
                objectiveai::functions::inventions::recursive::request::FunctionInventionRecursiveCreateParams {
                    remote: request.remote,
                    name: request.name.clone(),
                    github_token: request.github_token.clone(),
                    state: child_state,
                    provider: request.provider.clone(),
                    agent: request.agent.clone(),
                    agents: request.agents.clone(),
                    seed: request.seed,
                    stream: request.stream,
                    max_step_retries: request.max_step_retries,
                    mcp_server_authorization: request.mcp_server_authorization.clone(),
                },
            );

            child_streams.push(run_recursive(
                invention_client.clone(),
                ctx.clone(),
                child_request,
                id.clone(),
                created,
                object,
                choice_indexer.clone(),
                child_native,
            ));
        }

        // Merge all child streams, yield chunks immediately, and collect
        // child invention paths for placeholder replacement.
        let mut child_paths: Vec<objectiveai::functions::RemoteFunctionPath> = Vec::new();
        let mut merged = futures::stream::select_all(child_streams);
        while let Some(chunk) = merged.next().await {
            // Collect paths from child inventions as they complete.
            for invention in &chunk.inventions {
                if let Some(path) = &invention.inner.path {
                    child_paths.push(path.clone());
                }
            }
            yield chunk;
        }

        // All children are done. Replace placeholders on the root state
        // and re-publish the updated function.
        if child_paths.is_empty() || final_path.is_none() {
            return;
        }

        state.replace_placeholders(&child_paths);
        let function = match state.build_function() {
            Some(f) => f,
            None => return,
        };

        let name = state.name();
        let publish_files = crate::functions::inventions::extract_publish_files(&state, &function);
        let description = crate::functions::inventions::extract_description(&state);

        let (updated_path, publish_error) = match request.remote {
            objectiveai::functions::Remote::Filesystem => {
                match crate::functions::inventions::publish_filesystem(
                    &invention_client.filesystem_client, name, &publish_files,
                ) {
                    Ok(path) => (Some(path), None),
                    Err(e) => (None, Some(e)),
                }
            }
            objectiveai::functions::Remote::Github => {
                let token = request.github_token.as_deref().unwrap_or("");
                match crate::functions::inventions::publish_github(
                    &invention_client.github_client,
                    &invention_client.filesystem_client,
                    token, name, &description, &publish_files,
                ).await {
                    Ok(path) => (Some(path), None),
                    Err(e) => (None, Some(e)),
                }
            }
            objectiveai::functions::Remote::Mock => (None, None),
        };

        // Yield a final chunk with the updated state, function, and path.
        let inventions_errors = if publish_error.is_some() { Some(true) } else { None };
        yield RecursiveChunk {
            id: id.clone(),
            inventions: vec![RecursiveInventionChunk {
                index: choice_indexer.get(native_index),
                inner: FunctionInventionChunk {
                    id: id.clone(),
                    completions: vec![],
                    state: Some(state),
                    path: updated_path,
                    function: Some(function),
                    created,
                    object: object.into(),
                    usage: None,
                    error: publish_error.map(|msg| objectiveai::error::ResponseError {
                        code: 500,
                        message: serde_json::json!({
                            "kind": "publish",
                            "error": msg,
                        }),
                    }),
                },
            }],
            inventions_errors,
            created,
            object,
            usage: None,
        };
    })
}

