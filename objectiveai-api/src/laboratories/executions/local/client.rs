use std::sync::Arc;
use std::time;
use bollard::exec::CreateExecOptions;
use bollard::models::ContainerCreateBody;
use bollard::query_parameters::{CreateContainerOptionsBuilder, UploadToContainerOptionsBuilder};
use futures::{Stream, StreamExt};
use crate::ctx;
use crate::util::{ChoiceIndexer, StreamOnce};

use objectiveai::agent::completions::message::{Message, UserMessage, RichContent, RichContentPart};

type LaboratoryExecutionChunk =
    objectiveai::laboratories::executions::response::streaming::LaboratoryExecutionChunk;
type BuilderChunk =
    objectiveai::laboratories::executions::response::streaming::BuilderChunk;
type EvaluationChunk =
    objectiveai::laboratories::executions::response::streaming::EvaluationChunk;
type Object = objectiveai::laboratories::executions::response::streaming::Object;
type Params = objectiveai::laboratories::executions::request::LaboratoryExecutionCreateParams;

type Continuation<OPENROUTER, CLAUDEAGENTSDK, MOCK> =
    crate::agent::completions::Continuation<
        <OPENROUTER as crate::agent::completions::UpstreamClient<
            objectiveai::agent::openrouter::Agent, objectiveai::agent::openrouter::Continuation,
        >>::State,
        <CLAUDEAGENTSDK as crate::agent::completions::UpstreamClient<
            objectiveai::agent::claude_agent_sdk::Agent, objectiveai::agent::claude_agent_sdk::Continuation,
        >>::State,
        <MOCK as crate::agent::completions::UpstreamClient<
            objectiveai::agent::mock::Agent, objectiveai::agent::mock::Continuation,
        >>::State,
    >;

pub fn response_id(created: u64) -> String {
    let uuid = uuid::Uuid::new_v4();
    format!("lbexec-{}-{created}", uuid.simple())
}

/// Laboratory client that runs builder agents in local Docker containers
/// with the embedded objectiveai-mcp binary.
pub struct Client<CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, RETRG, RETRF, RETRM, CUSG, LUSG> {
    pub agent_client: Arc<
        crate::agent::completions::Client<
            CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, RETRG, RETRF, RETRM, CUSG,
        >,
    >,
    pub retrieve_router:
        Arc<crate::retrieval::retrieve::Router<RETRG, RETRF, RETRM, CTXEXT>>,
    pub usage_handler: Arc<LUSG>,
    pub viewer: Arc<crate::viewer::Client<CTXEXT>>,
}

/// Create a tar archive containing the MCP binary at the archive root.
fn mcp_tar(binary: &[u8]) -> Vec<u8> {
    let mut ar = tar::Builder::new(Vec::new());
    let mut header = tar::Header::new_gnu();
    header.set_size(binary.len() as u64);
    header.set_mode(0o755);
    header.set_cksum();
    ar.append_data(&mut header, "objectiveai-mcp", binary)
        .expect("failed to build tar archive");
    ar.into_inner().expect("failed to finalize tar archive")
}

/// Spawn a single builder container: create, start, upload MCP binary, and start the MCP server.
/// Returns the container ID.
async fn spawn_builder(
    docker: &bollard::Docker,
    image: &str,
    index: usize,
    mcp_tar: &[u8],
) -> Result<String, super::Error> {
    let container_name = format!("objectiveai-lab-builder-{index}");
    let options = CreateContainerOptionsBuilder::default()
        .name(container_name.as_str())
        .build();

    let config = ContainerCreateBody {
        image: Some(image.to_string()),
        cmd: Some(vec!["sleep".to_string(), "infinity".to_string()]),
        ..Default::default()
    };

    let container = docker
        .create_container(Some(options), config)
        .await
        .map_err(|e| super::Error::Docker(e.to_string()))?;

    docker
        .start_container(&container.id, None)
        .await
        .map_err(|e| super::Error::Docker(e.to_string()))?;

    let upload_options = UploadToContainerOptionsBuilder::default()
        .path("/")
        .build();

    docker
        .upload_to_container(
            &container.id,
            Some(upload_options),
            bollard::body_full(mcp_tar.to_vec().into()),
        )
        .await
        .map_err(|e| super::Error::Docker(e.to_string()))?;

    let exec_options = CreateExecOptions {
        cmd: Some(vec!["/objectiveai-mcp"]),
        attach_stdin: Some(true),
        attach_stdout: Some(true),
        attach_stderr: Some(true),
        ..Default::default()
    };

    let exec = docker
        .create_exec(&container.id, exec_options)
        .await
        .map_err(|e| super::Error::Docker(e.to_string()))?;

    let _start_result = docker
        .start_exec(&exec.id, None)
        .await
        .map_err(|e| super::Error::Docker(e.to_string()))?;

    Ok(container.id)
}

/// Add an MCP server address to an inline agent base.
fn inject_mcp_server(agent: &mut objectiveai::agent::InlineAgentBase, mcp_url: String) {
    let server = objectiveai::agent::McpServer {
        url: mcp_url,
        authorization: false,
    };
    match agent {
        objectiveai::agent::InlineAgentBase::Openrouter(b) => {
            b.mcp_servers.get_or_insert_with(Vec::new).push(server);
        }
        objectiveai::agent::InlineAgentBase::ClaudeAgentSdk(b) => {
            b.mcp_servers.get_or_insert_with(Vec::new).push(server);
        }
        objectiveai::agent::InlineAgentBase::Mock(_) => {}
    }
}

impl<CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, RETRG, RETRF, RETRM, CUSG, LUSG>
    Client<CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, RETRG, RETRF, RETRM, CUSG, LUSG>
where
    CTXEXT: ctx::ContextExt + Send + Sync + 'static,
    OPENROUTER: crate::agent::completions::UpstreamClient<
            objectiveai::agent::openrouter::Agent, objectiveai::agent::openrouter::Continuation,
        > + Send
        + Sync
        + 'static,
    CLAUDEAGENTSDK: crate::agent::completions::UpstreamClient<
            objectiveai::agent::claude_agent_sdk::Agent, objectiveai::agent::claude_agent_sdk::Continuation,
        > + Send
        + Sync
        + 'static,
    MOCK: crate::agent::completions::UpstreamClient<
            objectiveai::agent::mock::Agent, objectiveai::agent::mock::Continuation,
        > + Send
        + Sync
        + 'static,
    RETRG: crate::retrieval::retrieve::Client<CTXEXT> + Send + Sync + 'static,
    RETRF: crate::retrieval::retrieve::Client<CTXEXT> + Send + Sync + 'static,
    RETRM: crate::retrieval::retrieve::Client<CTXEXT> + Send + Sync + 'static,
    CUSG: crate::agent::completions::usage_handler::UsageHandler<CTXEXT> + Send + Sync + 'static,
    LUSG: crate::laboratories::executions::usage_handler::UsageHandler<CTXEXT> + Send + Sync + 'static,
{
    pub async fn create_streaming(
        self: Arc<Self>,
        ctx: ctx::Context<CTXEXT, impl crate::ctx::persistent_cache::PersistentCacheClient>,
        request: Arc<Params>,
    ) -> Result<
        impl Stream<Item = LaboratoryExecutionChunk> + Send + 'static,
        super::Error,
    > {
        // Timestamp and identify the execution — before any awaits
        let created = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let id = response_id(created);
        let object = Object::LaboratoryExecutionChunk;

        // Send begin to viewer
        self.viewer.send_laboratory_execution_begin(
            ctx.clone(),
            id.clone(),
            request.clone(),
        );

        // Helper: send error to viewer and return it
        let send_err = |e: super::Error| -> super::Error {
            self.viewer.send_laboratory_execution_error(
                ctx.clone(),
                id.clone(),
                objectiveai::error::ResponseError::from(&e),
            );
            e
        };

        if request.builder_agents.is_empty() {
            return Err(send_err(super::Error::NoBuilderAgents));
        }

        // Connect to Docker
        let docker = bollard::Docker::connect_with_local_defaults()
            .map_err(|e| send_err(super::Error::Docker(e.to_string())))?;

        let tar_bytes = mcp_tar(super::mcp_binary::MCP_BINARY);

        // Spawn containers and resolve agents concurrently — single await
        let docker_futs: Vec<_> = request
            .builder_agents
            .iter()
            .enumerate()
            .map(|(i, _)| spawn_builder(&docker, &request.docker_image, i, &tar_bytes))
            .collect();
        let resolve_futs: Vec<_> = request
            .builder_agents
            .iter()
            .map(|agent_ref| self.retrieve_router.get_agent(&ctx, agent_ref.clone()))
            .collect();
        let (container_ids, resolved_agents) = tokio::try_join!(
            futures::future::try_join_all(docker_futs),
            async {
                futures::future::try_join_all(resolve_futs)
                    .await
                    .map_err(|e| super::Error::AgentCompletion(e.to_string()))
            },
        )
        .map_err(&send_err)?;

        let mut inline_agents = Vec::with_capacity(request.builder_agents.len());
        for (i, agent_wf) in resolved_agents.into_iter().enumerate() {
            let mut agent_base = agent_wf.inline().inner.clone().into_base();

            // TODO: derive MCP URL from container networking
            let mcp_url = format!("http://{}:3000", container_ids[i]);
            inject_mcp_server(&mut agent_base, mcp_url);

            inline_agents.push(agent_base);
        }

        // Create agent completions for each builder concurrently
        let indexer = Arc::new(ChoiceIndexer::new(0));
        let agent_client = self.agent_client.clone();

        let streams: Vec<_> = inline_agents
            .into_iter()
            .enumerate()
            .map(|(native_index, agent_base)| {
                let agent_client = agent_client.clone();
                let ctx = ctx.clone();
                let request = request.clone();
                let indexer = indexer.clone();
                let id = id.clone();
                let container_index = native_index as u64;

                let agent_with_fallbacks = objectiveai::agent::InlineAgentBaseWithFallbacks {
                    inner: agent_base,
                    fallbacks: None,
                };
                let agent =
                    objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional::AgentBase(
                        agent_with_fallbacks,
                    );

                let params = Arc::new(
                    objectiveai::agent::completions::request::AgentCompletionCreateParams {
                        messages: request.builder_messages.clone(),
                        provider: request.provider.clone(),
                        agent,
                        response_format: None,
                        seed: request.seed,
                        stream: Some(true),
                        continuation: request.builder_continuation.clone(),
                    },
                );

                Box::pin(async_stream::stream! {
                    let stream_result = agent_client
                        .create_streaming(ctx, params, None, None, None, None, false)
                        .await;

                    match stream_result {
                        Ok(stream) => {
                            futures::pin_mut!(stream);
                            while let Some(item) = stream.next().await {
                                match item {
                                    crate::agent::completions::StreamItem::Chunk(chunk) => {
                                        let completion_index = indexer.get(native_index);
                                        yield LaboratoryExecutionChunk {
                                            id: id.clone(),
                                            builders: vec![BuilderChunk {
                                                index: completion_index,
                                                container_index,
                                                inner: chunk,
                                                error: None,
                                            }],
                                            evaluations: Vec::new(),
                                            error: None,
                                            created,
                                            object,
                                            usage: None,
                                        };
                                    }
                                    crate::agent::completions::StreamItem::State(_cont) => {
                                        // Continuation state — not used yet
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            let completion_index = indexer.get(native_index);
                            yield LaboratoryExecutionChunk {
                                id: id.clone(),
                                builders: vec![BuilderChunk {
                                    index: completion_index,
                                    container_index,
                                    inner: Default::default(),
                                    error: Some(objectiveai::error::ResponseError::from(&e)),
                                }],
                                evaluations: Vec::new(),
                                error: None,
                                created,
                                object,
                                usage: None,
                            };
                        }
                    }
                }) as std::pin::Pin<Box<dyn Stream<Item = LaboratoryExecutionChunk> + Send>>
            })
            .collect();

        let viewer_client = self.viewer.clone();
        let viewer_ctx = ctx.clone();
        let mut merged = futures::stream::select_all(streams);
        Ok(async_stream::stream! {
            let mut accumulated_usage = objectiveai::agent::completions::response::Usage::default();
            while let Some(chunk) = merged.next().await {
                for builder in &chunk.builders {
                    if let Some(u) = &builder.inner.usage {
                        accumulated_usage.push(u);
                    }
                }
                for evaluation in &chunk.evaluations {
                    if let Some(u) = &evaluation.inner.usage {
                        accumulated_usage.push(u);
                    }
                }
                viewer_client.send_laboratory_execution_continue(viewer_ctx.clone(), chunk.clone());
                yield chunk;
            }
            // Final chunk with accumulated usage
            let final_chunk = LaboratoryExecutionChunk {
                id,
                builders: Vec::new(),
                evaluations: Vec::new(),
                error: None,
                created,
                object,
                usage: Some(accumulated_usage),
            };
            viewer_client.send_laboratory_execution_continue(viewer_ctx.clone(), final_chunk.clone());
            yield final_chunk;
        })
    }

    /// Create a streaming evaluation for a single evaluation agent.
    ///
    /// Appends the evaluation schema to the messages, runs the agent completion,
    /// parses the response as `InputValue`, validates against the schema, and
    /// retries on error up to `max_evaluation_retries`.
    pub fn create_evaluation_streaming(
        self: Arc<Self>,
        ctx: ctx::Context<CTXEXT, impl crate::ctx::persistent_cache::PersistentCacheClient>,
        request: Arc<Params>,
        id: String,
        created: u64,
        object: Object,
        evaluation_index: u64,
        container_index: u64,
        agent: objectiveai::agent::InlineAgentBaseWithFallbacksOrRemoteCommitOptional,
    ) -> impl Stream<Item = LaboratoryExecutionChunk> + Send + 'static {
        let agent_client = self.agent_client.clone();
        let max_retries = request.max_evaluation_retries.unwrap_or(3);

        // Build the schema prompt suffix
        let schema_text = format!(
            "## evaluation schema\n\n{}",
            serde_json::to_string_pretty(&request.evaluation_output_schema).unwrap(),
        );

        // Inject schema into messages: append to last user message or create one
        let mut messages = request.evaluation_messages.clone();
        let mut injected = false;
        for msg in messages.iter_mut().rev() {
            if let Message::User(user) = msg {
                match &mut user.content {
                    RichContent::Text(t) => {
                        t.push_str("\n\n");
                        t.push_str(&schema_text);
                    }
                    RichContent::Parts(parts) => {
                        parts.push(RichContentPart::Text {
                            text: format!("\n\n{schema_text}"),
                        });
                    }
                }
                injected = true;
                break;
            }
        }
        if !injected {
            messages.push(Message::User(UserMessage {
                content: RichContent::Text(schema_text.clone()),
                name: None,
            }));
        }

        let params = Arc::new(
            objectiveai::agent::completions::request::AgentCompletionCreateParams {
                messages,
                provider: request.provider.clone(),
                agent,
                response_format: None,
                seed: request.seed,
                stream: Some(true),
                continuation: request.evaluation_continuation.clone(),
            },
        );

        async_stream::stream! {
            let mut continuation: Option<Continuation<OPENROUTER, CLAUDEAGENTSDK, MOCK>> = None;
            let mut retries = 0u32;

            loop {
                // Create agent completion stream
                let stream_result = agent_client
                    .create_streaming(
                        ctx.clone(),
                        params.clone(),
                        continuation.take(),
                        None,
                        None,
                        None,
                        false,
                    )
                    .await;

                let mut accumulated_chunk: Option<objectiveai::agent::completions::response::streaming::AgentCompletionChunk> = None;

                match stream_result {
                    Ok(stream) => {
                        futures::pin_mut!(stream);
                        while let Some(item) = stream.next().await {
                            match item {
                                crate::agent::completions::StreamItem::Chunk(chunk) => {
                                    match &mut accumulated_chunk {
                                        Some(acc) => acc.push(&chunk),
                                        None => accumulated_chunk = Some(chunk.clone()),
                                    }
                                    yield LaboratoryExecutionChunk {
                                        id: id.clone(),
                                        builders: Vec::new(),
                                        evaluations: vec![EvaluationChunk {
                                            index: evaluation_index,
                                            container_index,
                                            inner: chunk,
                                            output: None,
                                            error: None,
                                        }],
                                        error: None,
                                        created,
                                        object,
                                        usage: None,
                                    };
                                }
                                crate::agent::completions::StreamItem::State(cont) => {
                                    continuation = Some(cont);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        yield LaboratoryExecutionChunk {
                            id: id.clone(),
                            builders: Vec::new(),
                            evaluations: vec![EvaluationChunk {
                                index: evaluation_index,
                                container_index,
                                inner: Default::default(),
                                output: None,
                                error: Some(objectiveai::error::ResponseError::from(&e)),
                            }],
                            error: None,
                            created,
                            object,
                            usage: None,
                        };
                        break;
                    }
                }

                // Extract assistant content text from accumulated chunks
                let content_text = accumulated_chunk
                    .as_ref()
                    .and_then(|chunk| {
                        chunk.messages.iter().rev().find_map(|msg| {
                            if let objectiveai::agent::completions::response::streaming::MessageChunk::Assistant(asst) = msg {
                                asst.content.as_ref().map(|c| match c {
                                    RichContent::Text(t) => t.clone(),
                                    RichContent::Parts(parts) => parts
                                        .iter()
                                        .filter_map(|p| match p {
                                            RichContentPart::Text { text } => Some(text.as_str()),
                                            _ => None,
                                        })
                                        .collect::<Vec<_>>()
                                        .join(""),
                                })
                            } else {
                                None
                            }
                        })
                    })
                    .unwrap_or_default();

                // Parse as InputValue
                let parse_result: Result<objectiveai::functions::expression::InputValue, _> = {
                    let mut de = serde_json::Deserializer::from_str(&content_text);
                    serde_path_to_error::deserialize(&mut de)
                };

                match parse_result {
                    Ok(input_value) => {
                        // Validate against schema
                        let valid = request
                            .evaluation_output_schema
                            .validate_input(&input_value);

                        if valid {
                            // Yield final chunk with output
                            yield LaboratoryExecutionChunk {
                                id: id.clone(),
                                builders: Vec::new(),
                                evaluations: vec![EvaluationChunk {
                                    index: evaluation_index,
                                    container_index,
                                    inner: Default::default(),
                                    output: Some(input_value),
                                    error: None,
                                }],
                                error: None,
                                created,
                                object,
                                usage: None,
                            };
                            break;
                        }

                        // Schema validation failed
                        let err = super::Error::EvaluationOutputSchemaMismatch;
                        if retries >= max_retries {
                            yield LaboratoryExecutionChunk {
                                id: id.clone(),
                                builders: Vec::new(),
                                evaluations: vec![EvaluationChunk {
                                    index: evaluation_index,
                                    container_index,
                                    inner: Default::default(),
                                    output: None,
                                    error: Some(objectiveai::error::ResponseError::from(&err)),
                                }],
                                error: None,
                                created,
                                object,
                                usage: None,
                            };
                            break;
                        }

                        // Retry with error message
                        let retry_msg = format!(
                            "{}\n\n## error\n\nevaluation output does not match schema",
                            schema_text,
                        );
                        if let Some(ref mut cont) = continuation {
                            cont.push_user_message(UserMessage {
                                content: RichContent::Text(retry_msg),
                                name: None,
                            });
                        }
                        retries += 1;
                    }
                    Err(parse_err) => {
                        // Parse failed
                        let err = super::Error::EvaluationOutputParse(parse_err.to_string());
                        if retries >= max_retries {
                            yield LaboratoryExecutionChunk {
                                id: id.clone(),
                                builders: Vec::new(),
                                evaluations: vec![EvaluationChunk {
                                    index: evaluation_index,
                                    container_index,
                                    inner: Default::default(),
                                    output: None,
                                    error: Some(objectiveai::error::ResponseError::from(&err)),
                                }],
                                error: None,
                                created,
                                object,
                                usage: None,
                            };
                            break;
                        }

                        // Retry with parse error
                        let retry_msg = format!(
                            "{}\n\n## error\n\n{}",
                            schema_text, parse_err,
                        );
                        if let Some(ref mut cont) = continuation {
                            cont.push_user_message(UserMessage {
                                content: RichContent::Text(retry_msg),
                                name: None,
                            });
                        }
                        retries += 1;
                    }
                }
            }
        }
    }
}

impl<CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, RETRG, RETRF, RETRM, CUSG, LUSG>
    crate::laboratories::executions::LaboratoryClient<CTXEXT>
    for Client<CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, RETRG, RETRF, RETRM, CUSG, LUSG>
where
    CTXEXT: ctx::ContextExt + Send + Sync + 'static,
    OPENROUTER: crate::agent::completions::UpstreamClient<
            objectiveai::agent::openrouter::Agent, objectiveai::agent::openrouter::Continuation,
        > + Send
        + Sync
        + 'static,
    CLAUDEAGENTSDK: crate::agent::completions::UpstreamClient<
            objectiveai::agent::claude_agent_sdk::Agent, objectiveai::agent::claude_agent_sdk::Continuation,
        > + Send
        + Sync
        + 'static,
    MOCK: crate::agent::completions::UpstreamClient<
            objectiveai::agent::mock::Agent, objectiveai::agent::mock::Continuation,
        > + Send
        + Sync
        + 'static,
    RETRG: crate::retrieval::retrieve::Client<CTXEXT> + Send + Sync + 'static,
    RETRF: crate::retrieval::retrieve::Client<CTXEXT> + Send + Sync + 'static,
    RETRM: crate::retrieval::retrieve::Client<CTXEXT> + Send + Sync + 'static,
    CUSG: crate::agent::completions::usage_handler::UsageHandler<CTXEXT> + Send + Sync + 'static,
    LUSG: crate::laboratories::executions::usage_handler::UsageHandler<CTXEXT> + Send + Sync + 'static,
{
    type Error = super::Error;

    fn create_unary_handle_usage(
        self: Arc<Self>,
        ctx: ctx::Context<CTXEXT, impl crate::ctx::persistent_cache::PersistentCacheClient>,
        request: Arc<Params>,
    ) -> impl std::future::Future<
        Output = Result<
            objectiveai::laboratories::executions::response::unary::LaboratoryExecution,
            super::Error,
        >,
    > + Send {
        async move {
            let mut aggregate: Option<LaboratoryExecutionChunk> = None;
            let mut stream =
                self.create_streaming_handle_usage(ctx, request).await?;
            while let Some(chunk) = stream.next().await {
                match &mut aggregate {
                    Some(aggregate) => aggregate.push(&chunk),
                    None => aggregate = Some(chunk),
                }
            }
            Ok(aggregate.unwrap().into())
        }
    }

    fn create_streaming_handle_usage(
        self: Arc<Self>,
        ctx: ctx::Context<CTXEXT, impl crate::ctx::persistent_cache::PersistentCacheClient>,
        request: Arc<Params>,
    ) -> impl std::future::Future<
        Output = Result<
            impl Stream<Item = LaboratoryExecutionChunk> + Send + Unpin + 'static,
            super::Error,
        >,
    > + Send {
        async move {
            let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
            tokio::spawn(async move {
                let mut aggregate: Option<LaboratoryExecutionChunk> = None;
                let stream = match self
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
                    if tx.send(Ok(chunk)).is_err() {
                        ctx.cancel();
                    }
                }
                drop(stream);
                drop(tx);
                let response: objectiveai::laboratories::executions::response::unary::LaboratoryExecution =
                    aggregate.unwrap().into();
                if response.any_usage() {
                    self.usage_handler
                        .handle_usage(ctx, request, response)
                        .await;
                }
            });
            let mut stream =
                tokio_stream::wrappers::UnboundedReceiverStream::new(rx);
            match stream.next().await {
                Some(Ok(chunk)) => {
                    Ok(StreamOnce::new(chunk).chain(stream.map(Result::unwrap)))
                }
                Some(Err(e)) => Err(e),
                None => unreachable!(),
            }
        }
    }
}
