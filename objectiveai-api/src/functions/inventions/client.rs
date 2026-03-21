use crate::{ctx, util::StreamOnce};
use futures::{Stream, StreamExt};
use objectiveai::error::StatusError;
use std::{
    pin::Pin,
    sync::{Arc, Mutex},
    time,
};

type FunctionInventionChunk =
    objectiveai::functions::inventions::response::streaming::FunctionInventionChunk;
type InventionAgentCompletionChunk =
    objectiveai::functions::inventions::response::streaming::AgentCompletionChunk;
type Object = objectiveai::functions::inventions::response::streaming::Object;
type Params = objectiveai::functions::inventions::Params;
type State = objectiveai::functions::inventions::State;

use objectiveai::functions::inventions::InventionState;

/// Generates a unique response ID for Function inventions.
pub fn invention_response_id(created: u64) -> String {
    let uuid = uuid::Uuid::new_v4();
    format!("fninv1-{}-{}", uuid.simple(), created)
}

/// Maximum total name length in bytes.
const MAX_NAME_LEN: usize = 100;
/// Maximum name length when no valid path segment exists.
/// Leaves room for `-` (1 byte) + base62 path segment (up to 22 bytes).
const MAX_NAME_LEN_WITHOUT_PATH: usize = 77;

/// Validates the invention name length constraints.
///
/// - Must be at most 100 bytes total.
/// - If the name does not already contain a valid base62 path segment
///   (the part after the last `-`), it must be at most 77 bytes to leave
///   room for child path segments (`-` + up to 22 bytes of base62).
fn validate_name(name: &str) -> Result<(), super::Error> {
    let len = name.len();
    if len > MAX_NAME_LEN {
        return Err(super::Error::InvalidName(format!(
            "name is {} bytes, maximum is {}",
            len, MAX_NAME_LEN,
        )));
    }
    let has_valid_path = name
        .rsplit_once('-')
        .and_then(|(_, last)| objectiveai::functions::inventions::path::b62_to_path::<u64>(last).ok())
        .is_some();
    if !has_valid_path && len > MAX_NAME_LEN_WITHOUT_PATH {
        return Err(super::Error::InvalidName(format!(
            "name is {} bytes without a path segment, maximum is {} \
             (must leave room for child path `-` + up to 22 bytes)",
            len, MAX_NAME_LEN_WITHOUT_PATH,
        )));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/// Client for inventing Functions.
///
/// Orchestrates the multi-step invention flow: essay, input schema,
/// essay tasks, tasks, description, and readme generation.
pub struct Client<CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, RETRG, RETRF, RETRM, CUSG, IUSG, FFNG, FFNF, FFNM> {
    pub agent_client: Arc<
        crate::agent::completions::Client<
            CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, RETRG, RETRF, RETRM, CUSG,
        >,
    >,
    pub github_client: Arc<crate::github::Client>,
    pub filesystem_client: Arc<crate::filesystem::Client>,
    pub retrieve_router:
        Arc<crate::retrieval::retrieve::Router<FFNG, FFNF, FFNM, CTXEXT>>,
    pub usage_handler: Arc<IUSG>,
    pub persist: bool,
}

impl<CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, RETRG, RETRF, RETRM, CUSG, IUSG, FFNG, FFNF, FFNM>
    Client<CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, RETRG, RETRF, RETRM, CUSG, IUSG, FFNG, FFNF, FFNM>
{
    pub fn new(
        agent_client: Arc<
            crate::agent::completions::Client<
                CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, RETRG, RETRF, RETRM, CUSG,
            >,
        >,
        github_client: Arc<crate::github::Client>,
        filesystem_client: Arc<crate::filesystem::Client>,
        retrieve_router: Arc<
            crate::retrieval::retrieve::Router<FFNG, FFNF, FFNM, CTXEXT>,
        >,
        usage_handler: Arc<IUSG>,
        persist: bool,
    ) -> Self {
        Self {
            agent_client,
            github_client,
            filesystem_client,
            retrieve_router,
            usage_handler,
            persist,
        }
    }
}

type Continuation<OPENROUTER, CLAUDEAGENTSDK, MOCK> =
    crate::agent::completions::Continuation<
        <OPENROUTER as crate::agent::completions::UpstreamClient<
            objectiveai::agent::openrouter::Agent,
        >>::State,
        <CLAUDEAGENTSDK as crate::agent::completions::UpstreamClient<
            objectiveai::agent::claude_agent_sdk::Agent,
        >>::State,
        <MOCK as crate::agent::completions::UpstreamClient<
            objectiveai::agent::mock::Agent,
        >>::State,
    >;

impl<CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, RETRG, RETRF, RETRM, CUSG, IUSG, FFNG, FFNF, FFNM>
    Client<CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, RETRG, RETRF, RETRM, CUSG, IUSG, FFNG, FFNF, FFNM>
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
    RETRG: crate::retrieval::retrieve::Client<CTXEXT>,
    RETRF: crate::retrieval::retrieve::Client<CTXEXT>,
    RETRM: crate::retrieval::retrieve::Client<CTXEXT>,
    CUSG: crate::agent::completions::usage_handler::UsageHandler<CTXEXT> + Send + Sync + 'static,
    IUSG: super::usage_handler::UsageHandler<CTXEXT> + Send + Sync + 'static,
    FFNG: crate::retrieval::retrieve::Client<CTXEXT> + Send + Sync + 'static,
    FFNF: crate::retrieval::retrieve::Client<CTXEXT> + Send + Sync + 'static,
    FFNM: crate::retrieval::retrieve::Client<CTXEXT> + Send + Sync + 'static,
{
    pub async fn create_unary_handle_usage(
        self: Arc<Self>,
        ctx: ctx::Context<CTXEXT>,
        request: Arc<
            objectiveai::functions::inventions::request::FunctionInventionCreateParams,
        >,
    ) -> Result<
        objectiveai::functions::inventions::response::unary::FunctionInvention,
        super::Error,
    > {
        let mut aggregate: Option<FunctionInventionChunk> = None;
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

    pub async fn create_streaming_handle_usage(
        self: Arc<Self>,
        ctx: ctx::Context<CTXEXT>,
        request: Arc<
            objectiveai::functions::inventions::request::FunctionInventionCreateParams,
        >,
    ) -> Result<
        impl Stream<Item = FunctionInventionChunk> + Send + Unpin + 'static,
        super::Error,
    > {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        tokio::spawn(async move {
            let mut aggregate: Option<FunctionInventionChunk> = None;
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
                let _ = tx.send(Ok(chunk));
            }
            drop(stream);
            drop(tx);
            if let Some(aggregate) = aggregate {
                if aggregate.usage.as_ref().is_some_and(
                    objectiveai::agent::completions::response::Usage::any_usage,
                ) {
                    self.usage_handler
                        .handle_usage(ctx, request, aggregate.into())
                        .await;
                }
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

    pub async fn create_streaming(
        self: Arc<Self>,
        ctx: ctx::Context<CTXEXT>,
        request: Arc<
            objectiveai::functions::inventions::request::FunctionInventionCreateParams,
        >,
    ) -> Result<
        impl Stream<Item = FunctionInventionChunk> + Send + 'static,
        super::Error,
    > {
        // Validate params before starting.
        let params = match &request.state {
            objectiveai::functions::inventions::state::ParamsState::AlphaScalarBranch(s) => &s.params,
            objectiveai::functions::inventions::state::ParamsState::AlphaScalarLeaf(s) => &s.params,
            objectiveai::functions::inventions::state::ParamsState::AlphaVectorBranch(s) => &s.params,
            objectiveai::functions::inventions::state::ParamsState::AlphaVectorLeaf(s) => &s.params,
            objectiveai::functions::inventions::state::ParamsState::AlphaScalar(s) => &s.params,
            objectiveai::functions::inventions::state::ParamsState::AlphaVector(s) => &s.params,
        };
        params.validate().map_err(super::Error::InvalidState)?;
        validate_name(&params.name)?;

        // Validate depth matches variant.
        let is_leaf = matches!(
            &request.state,
            objectiveai::functions::inventions::state::ParamsState::AlphaScalarLeaf(_)
            | objectiveai::functions::inventions::state::ParamsState::AlphaVectorLeaf(_)
        );
        let is_branch = matches!(
            &request.state,
            objectiveai::functions::inventions::state::ParamsState::AlphaScalarBranch(_)
            | objectiveai::functions::inventions::state::ParamsState::AlphaVectorBranch(_)
        );
        if is_leaf && params.depth > 0 {
            return Err(super::Error::InvalidState(
                format!("leaf state requires depth=0, got depth={}", params.depth),
            ));
        }
        if is_branch && params.depth == 0 {
            return Err(super::Error::InvalidState(
                "branch state requires depth>0, got depth=0".to_string(),
            ));
        }

        // Pre-flight: validate remote, token, and name.
        self.check_preflight(&ctx, &request, &params.name).await?;

        let created = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let id = invention_response_id(created);
        let mut state = request.state.clone().route();
        state.set_checker_seed(request.seed);

        // Validate predicted tasks_length against params bounds.
        if let Some(tasks_length) = state.tasks_length() {
            let p = state.params();
            let (min, max) = match &state {
                State::AlphaScalarBranch(_) | State::AlphaVectorBranch(_) => {
                    (p.min_branch_width, p.max_branch_width)
                }
                State::AlphaScalarLeaf(_) | State::AlphaVectorLeaf(_) => {
                    (p.min_leaf_width, p.max_leaf_width)
                }
            };
            if tasks_length < min || tasks_length > max {
                return Err(super::Error::InvalidState(format!(
                    "tasks_length {} is outside bounds [{}, {}]",
                    tasks_length, min, max,
                )));
            }
        }

        // If the initial state has tasks, fetch all referenced child functions
        // and validate the initial state against them.
        let children = if let Some(full_fn) = state.build_function() {
            let children = self.retrieve_router.get_function_recursive(&ctx, full_fn).await
                .map_err(super::Error::FunctionFetch)?;
            Some(children)
        } else {
            None
        };
        state
            .validate_initial_state(children.as_ref())
            .map_err(super::Error::InvalidState)?;
        let agent_client = self.agent_client.clone();
        let github_client = self.github_client.clone();
        let filesystem_client = self.filesystem_client.clone();
        let persist = self.persist;

        let stream: Pin<Box<dyn Stream<Item = FunctionInventionChunk> + Send>> =
            match state {
                State::AlphaScalarBranch(s) => {
                    run_all_steps(s, agent_client, github_client, filesystem_client, ctx, request, id, created, persist)
                }
                State::AlphaScalarLeaf(s) => {
                    run_all_steps(s, agent_client, github_client, filesystem_client, ctx, request, id, created, persist)
                }
                State::AlphaVectorBranch(s) => {
                    run_all_steps(s, agent_client, github_client, filesystem_client, ctx, request, id, created, persist)
                }
                State::AlphaVectorLeaf(s) => {
                    run_all_steps(s, agent_client, github_client, filesystem_client, ctx, request, id, created, persist)
                }
            };

        Ok(stream)
    }

    /// Pre-flight checks before starting the invention flow.
    ///
    /// - If remote is GitHub: validates that `github_token` is present, valid,
    ///   and has permissions to create repos, push, and edit descriptions.
    /// - If `overwrite` is not true: checks that the name doesn't already
    ///   exist on the target remote.
    async fn check_preflight(
        &self,
        ctx: &ctx::Context<CTXEXT>,
        request: &objectiveai::functions::inventions::request::FunctionInventionCreateParams,
        name: &str,
    ) -> Result<(), super::Error> {
        let remote = match &request.remote {
            Some(r) => r,
            None => return Ok(()),
        };

        // GitHub remote: validate token and check permissions.
        if matches!(remote, objectiveai::Remote::Github) {
            let scopes = self
                .github_client
                .validate_token(ctx)
                .await?;

            // The `repo` scope grants create, push, and description edit.
            // Fine-grained tokens use different headers, so also accept
            // an empty scopes list (fine-grained tokens don't return
            // x-oauth-scopes). Classic tokens must have `repo`.
            if !scopes.is_empty() && !scopes.iter().any(|s| s == "repo" || s == "public_repo") {
                return Err(super::Error::GithubTokenMissingPermissions(
                    format!(
                        "Token must have 'repo' or 'public_repo' scope. Found: [{}]",
                        scopes.join(", "),
                    ),
                ));
            }
        }

        // Check name existence (skip if overwrite is true).
        if request.overwrite == Some(true) {
            return Ok(());
        }

        let exists = match remote {
            objectiveai::Remote::Github => {
                let (owner, repo) = if let Some((o, r)) = name.split_once('/') {
                    (o, r)
                } else {
                    // Cannot check without owner; skip.
                    return Ok(());
                };
                self.github_client
                    .repository_exists(ctx, owner, repo)
                    .await?
            }
            objectiveai::Remote::Filesystem => {
                let (owner, repo) = if let Some((o, r)) = name.split_once('/') {
                    (o, r)
                } else {
                    return Ok(());
                };
                self.filesystem_client.repository_exists(crate::retrieval::Kind::Functions, owner, repo)
            }
            objectiveai::Remote::Mock => false,
        };

        if exists {
            return Err(super::Error::NameAlreadyExists(name.to_string()));
        }

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Step orchestration
// ---------------------------------------------------------------------------

fn run_all_steps<T, CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, RETRG, RETRF, RETRM, CUSG>(
    state_val: T,
    agent_client: Arc<
        crate::agent::completions::Client<
            CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, RETRG, RETRF, RETRM, CUSG,
        >,
    >,
    github_client: Arc<crate::github::Client>,
    filesystem_client: Arc<crate::filesystem::Client>,
    ctx: ctx::Context<CTXEXT>,
    request: Arc<objectiveai::functions::inventions::request::FunctionInventionCreateParams>,
    id: String,
    created: u64,
    persist: bool,
) -> Pin<Box<dyn Stream<Item = FunctionInventionChunk> + Send>>
where
    T: InventionState,
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
    RETRG: crate::retrieval::retrieve::Client<CTXEXT>,
    RETRF: crate::retrieval::retrieve::Client<CTXEXT>,
    RETRM: crate::retrieval::retrieve::Client<CTXEXT>,
    CUSG: crate::agent::completions::usage_handler::UsageHandler<CTXEXT> + Send + Sync + 'static,
{
    Box::pin(async_stream::stream! {
        let state = Arc::new(Mutex::new(state_val));
        let params = T::params(&state);
        let is_scalar = T::is_scalar();
        let object = T::object();
        let tasks_str = tasks_str(&params);

        let state_chunk = |state: &Arc<Mutex<T>>, id: &str, created, object| {
            FunctionInventionChunk {
                id: id.to_string(),
                completions: vec![],
                state: Some(state.lock().unwrap().clone().into_state()),
                path: None,
                function: None,
                created,
                object,
                usage: None,
                error: None,
            }
        };

        // Continuation carried between steps.
        let mut continuation: Option<
            Continuation<OPENROUTER, CLAUDEAGENTSDK, MOCK>,
        > = None;
        // Completion index incremented across all steps.
        let mut completion_index: u64 = 0;
        // Accumulated usage across all steps.
        let mut accumulated_usage = objectiveai::agent::completions::response::Usage::default();

        // Initial state
        yield state_chunk(&state, &id, created, object);

        let mut errored = false;

        // Step 1: Essay
        let essay_validate = Arc::new({ let s = state.clone(); move || T::validate_essay(&s) });
        if essay_validate().is_err() {
        errored = false;
        let essay_prompt = if is_scalar {
            format!(
                "You are an inventor creating a new ObjectiveAI Function. \
                Write a non-technical essay describing the Scalar Function you are building. \
                Explore the purpose, input, and use-cases of the function in detail. \
                Explore the qualities and values that must be evaluated for the input. \
                There should be {tasks_str} qualities or values. \
                This essay will guide the development of the Scalar Function and underpins its philosophy. \
                Read the Spec first.",
            )
        } else {
            format!(
                "You are an inventor creating a new ObjectiveAI Function. \
                Write a non-technical essay describing the Vector Function you are building. \
                Explore the purpose, inputs, and use-cases of the function in detail. \
                Explore the qualities and values that must be evaluated in order to \
                properly rank items relative to one another. \
                There should be {tasks_str} qualities or values. \
                This essay will guide the development of the Vector Function and underpins its philosophy. \
                Read the Spec first.",
            )
        };
        let mut step = run_step(
            agent_client.clone(), ctx.clone(), request.clone(),
            essay_prompt, T::essay_tools(&state),
            essay_validate,
            id.clone(), created, object, continuation.take(), completion_index,
        );
        while let Some(output) = step.next().await {
            match output {
                StepOutput::Chunk(chunk) => {
                    errored = chunk.error.is_some();
                    for c in &chunk.completions {
                        if let Some(u) = &c.inner.usage {
                            accumulated_usage.push(u);
                        }
                    }
                    yield chunk;
                }
                StepOutput::Continuation(c) => { continuation = Some(c); }
                StepOutput::CompletionIndex(i) => { completion_index = i; }
            }
        }
        if errored {
            yield FunctionInventionChunk {
                id: id.to_string(), completions: vec![], state: None,
                path: None, function: None, created, object,
                usage: Some(accumulated_usage), error: None,
            };
            return;
        }
        }
        yield state_chunk(&state, &id, created, object);

        // Step 2: Input Schema
        let input_schema_validate = Arc::new({ let s = state.clone(); move || T::validate_input_schema(&s) });
        if input_schema_validate().is_err() {
        let input_schema_prompt = if is_scalar {
            "Create the InputSchema for your Scalar Function. \
            Ensure that it adheres to the specifications outlined in your Spec \
            and is consistent with the essay you wrote describing your function. \
            If handling multimodal content, use `type`: `image`, `audio`, `video`, or `file` depending. \
            Multimodal types exist in addition to common primitives (e.g. `string`, `array`, etc)".to_string()
        } else {
            "Create the InputSchema for your Vector Function. \
            Ensure that it adheres to the specifications outlined in your Spec \
            and is consistent with the essay you wrote describing your function. \
            If handling multimodal content, use `type`: `image`, `audio`, `video`, or `file` depending. \
            Multimodal types exist in addition to common primitives (e.g. `string`, `array`, etc)".to_string()
        };
        errored = false;
        let mut step = run_step(
            agent_client.clone(), ctx.clone(), request.clone(),
            input_schema_prompt, T::input_schema_tools(&state),
            input_schema_validate,
            id.clone(), created, object, continuation.take(), completion_index,
        );
        while let Some(output) = step.next().await {
            match output {
                StepOutput::Chunk(chunk) => {
                    errored = chunk.error.is_some();
                    for c in &chunk.completions {
                        if let Some(u) = &c.inner.usage {
                            accumulated_usage.push(u);
                        }
                    }
                    yield chunk;
                }
                StepOutput::Continuation(c) => { continuation = Some(c); }
                StepOutput::CompletionIndex(i) => { completion_index = i; }
            }
        }
        if errored {
            yield FunctionInventionChunk {
                id: id.to_string(), completions: vec![], state: None,
                path: None, function: None, created, object,
                usage: Some(accumulated_usage), error: None,
            };
            return;
        }
        }
        yield state_chunk(&state, &id, created, object);

        // Step 3: Essay Tasks
        let essay_tasks_validate = Arc::new({ let s = state.clone(); move || T::validate_essay_tasks(&s) });
        if essay_tasks_validate().is_err() {
        let essay_tasks_prompt = format!(
            "Write EssayTasks listing and describing the key tasks the Function must \
            perform in order to fulfill the quality and value evaluations defined within \
            the essay. Each task is a non-technical plain language description of a task \
            which will go into the function's `tasks` array. There should be {tasks_str} tasks. \
            Read the Spec and Essay first.",
        );
        errored = false;
        let mut step = run_step(
            agent_client.clone(), ctx.clone(), request.clone(),
            essay_tasks_prompt, T::essay_tasks_tools(&state),
            essay_tasks_validate,
            id.clone(), created, object, continuation.take(), completion_index,
        );
        while let Some(output) = step.next().await {
            match output {
                StepOutput::Chunk(chunk) => {
                    errored = chunk.error.is_some();
                    for c in &chunk.completions {
                        if let Some(u) = &c.inner.usage {
                            accumulated_usage.push(u);
                        }
                    }
                    yield chunk;
                }
                StepOutput::Continuation(c) => { continuation = Some(c); }
                StepOutput::CompletionIndex(i) => { completion_index = i; }
            }
        }
        if errored {
            yield FunctionInventionChunk {
                id: id.to_string(), completions: vec![], state: None,
                path: None, function: None, created, object,
                usage: Some(accumulated_usage), error: None,
            };
            return;
        }
        }
        yield state_chunk(&state, &id, created, object);

        // Step 4: Tasks (Body)
        let depth_propagation = if params.depth > 1 {
            " The sub-function will also have its own sub-functions. \
            The spec should include any instructions that should also be \
            propagated down to the child agent's own child agents, if any are needed."
        } else {
            ""
        };
        let tasks_prompt = if is_scalar {
            if params.depth > 0 {
                format!(
                    "Create the Tasks for your Scalar Function.\n\n\
                    ## Task Structure\n\n\
                    Create {tasks_str} placeholder tasks based on your EssayTasks. \
                    Each task defines a sub-function which will be automatically invented \
                    after you finish. Some tasks may have the same `input_schema` as the \
                    parent, and some may contain only a subset so as to evaluate a specific \
                    aspect of the input.\n\n\
                    **TaskSpec:**\n\
                    First, write a detailed `spec` for the task, describing what the \
                    sub-function should evaluate. This is a plain language description \
                    that will guide the child agent inventing the sub-function.{depth_propagation}\n\n\
                    **Task Fields:**\n\
                    - `name` — the sub-function's name\n\
                    - `spec` — detailed description of what the sub-function should evaluate\n\
                    - `input_schema` — the sub-function's input schema\n\
                    - `skip` — optional conditional skip expression, typically used to skip \
                    tasks which evaluate some optional field on the parent input\n\
                    - `input` — expression deriving the task input from the parent input\n\n\
                    ## Expression Context\n\n\
                    - `input` — the parent function's input\n\n\
                    ## Finishing\n\n\
                    1. Use CheckFunction to validate — fix any errors and retry until it passes\n\
                    2. Re-read the Spec. It is the universal source of truth — never contradict it.",
                )
            } else {
                format!(
                    "Create the Tasks for your Scalar Function.\n\n\
                    ## Task Structure\n\n\
                    Create {tasks_str} vector completion tasks based on your EssayTasks. \
                    Each task defines a prompt for an LLM as well as possible responses \
                    for the assistant to reply with. The ObjectiveAI system will return a \
                    vector of scores evaluating which response the LLM is most likely to \
                    reply with. These probabilities form the fundamental basis for how the \
                    Function scores the input.\n\n\
                    ### Messages\n\n\
                    `messages` is a Starlark expression producing the conversation prompt. \
                    Each message contains a role and an array of content parts. Typically, \
                    messages will be a single user message containing context from the input.\n\n\
                    ### Responses\n\n\
                    `responses` is an array of potential responses the LLM could reply with. \
                    Each response is an array of content parts. The responses are \
                    fixed potential replies. Each response automatically corresponds to a score \
                    with earlier responses being a low score, and later responses being a high \
                    score.\n\n\
                    ## Structure\n\n\
                    Be clever in how you structure `messages` and `responses`. Do not ask \
                    the LLM to directly score the input. Instead, make `responses` into real \
                    responses that an assistant would actually reply with in a conversation. \
                    For example, if asking for the quality of a joke, the message could be \
                    'How funny is this joke: {{joke}}?' and the responses could be 'not funny at all', \
                    'pretty funny', and 'hilarious'.\n\n\
                    ### Multimodal Content\n\n\
                    Multimodal content parts can be used in `messages` and are derived from the input. \
                    Never use `str()` on multimodal content — this breaks the system and makes \
                    it unintelligible to the LLM, ruining the scores.\n\n\
                    ### Key Design Principles\n\n\
                    - Some tasks may score a subset of the parent input. Other tasks may score \
                    the entire parent input. Some tasks may contain partial context, and others \
                    may contain full context.\n\
                    - Tasks should not be identical to each other. They should vary — be creative \
                    in how they vary, feel free to use multiple messages in some cases.\n\
                    - `skip` expressions conditionally skip tasks. This is typically used to skip \
                    tasks which use some optional field(s) on the parent input.\n\n\
                    ## Expression Context\n\n\
                    - `input` — the function's input\n\n\
                    ## Finishing\n\n\
                    1. Use CheckFunction to validate — fix any errors and retry until it passes\n\
                    2. Re-read the Spec. It is the universal source of truth — never contradict it.",
                )
            }
        } else if params.depth > 0 {
            format!(
                "Create the Tasks for your Vector Function.\n\n\
                ## Task Structure\n\n\
                Create {tasks_str} placeholder tasks based on your EssayTasks. \
                Each task defines a sub-function which will be automatically invented \
                after you finish. Some tasks may have the same `input_schema` as the \
                parent, and some may contain only a subset so as to evaluate a specific \
                aspect of the input.\n\
                You can mix two types of placeholder tasks:\n\
                - **Vector sub-functions** (`placeholder.alpha.vector.function`): Rank the \
                input items provided to the task relative to each other.\n\
                - **Scalar sub-functions** (`placeholder.alpha.scalar.function`): Score \
                individual items.\n\n\
                **TaskSpec:**\n\
                First, write a detailed `spec` for the task, describing what the \
                sub-function should evaluate. This is a plain language description \
                that will guide the child agent inventing the sub-function.{depth_propagation}\n\n\
                **Task Fields:**\n\
                - `name` — the sub-function's name\n\
                - `spec` — detailed description of what the sub-function should evaluate\n\
                - `input_schema` — the sub-function's input schema\n\
                - `skip` — optional conditional skip expression, typically used to skip \
                tasks which evaluate some optional field on the parent input\n\
                - `input` — expression deriving the task input from the parent input\n\n\
                ## Expression Context\n\n\
                - `input` — the parent function's input\n
                - `map` - only present in scalar sub-functions, the index of the item being scored\n\n\
                ## Finishing\n\n\
                1. Use CheckFunction to validate — fix any errors and retry until it passes\n\
                2. Re-read the Spec. It is the universal source of truth — never contradict it.",
            )
        } else {
            format!(
                "Create the Tasks for your Vector Function.\n\n\
                ## Task Structure\n\n\
                Create {tasks_str} vector completion tasks based on your EssayTasks. \
                Each task defines a prompt for an LLM as well as possible responses \
                for the assistant to reply with. The ObjectiveAI system will return a \
                vector of scores evaluating which response the LLM is most likely to \
                reply with. These probabilities form the fundamental basis for how the \
                Function ranks items.\n\n\
                ### Messages\n\n\
                `messages` is a Starlark expression producing the conversation prompt. \
                Each message contains a role and an array of content parts. Typically, \
                messages will be a single user message. Sometimes it is a fixed message. \
                Other times, it contains context from the input. But it never contains \
                the items to be ranked.\n\n\
                ### Responses\n\n\
                `responses` is a Starlark expression producing an array of potential \
                responses the LLM could reply with. Each response is an array of content parts.\n\n\
                ## Structure\n\n\
                Be clever in how you structure `messages` and `responses`. Do not ask \
                the LLM to directly evaluate items. Instead, make the items into real \
                responses that an assistant would actually reply with in a conversation. \
                For messages, do not structure it like 'Which item is best?' Instead, \
                structure it like 'What would a good item look like?' and make the \
                responses the items being ranked. If ranking search results, for example, \
                the message would be the search query, and the responses would be the \
                search results as-is.\n\n\
                ### Multimodal Content\n\n\
                Multimodal content parts can be used in both `messages` and `responses`. \
                Put contextual multimodal content in `messages`, and put multimodal \
                content which is being ranked into `responses`. Never use `str()` on \
                multimodal content — this breaks the system and makes it unintelligible \
                to the LLM, ruining the rankings.\n\n\
                ### Key Design Principles\n\n\
                - Some tasks may rank a subset of the parent input. Other tasks may rank \
                the entire parent input. Some tasks may contain partial context, and others \
                may contain full context. Tasks should not be identical to each other. They \
                should vary — be creative in how they vary, feel free to use multiple messages \
                in some cases.\n\
                - `skip` expressions conditionally skip tasks. This is typically used to skip \
                tasks which use some optional field(s) on the parent input.\n\
                - Ensure that each task ranks items in the same order. This is critical for \
                the ObjectiveAI system to be able to combine the rankings from different tasks \
                together into a single ranking.\n\n\
                ## Expression Context\n\n\
                - `input` — the function's input\n\n\
                ## Finishing\n\n\
                1. Use CheckFunction to validate — fix any errors and retry until it passes\n\
                2. Re-read the Spec. It is the universal source of truth — never contradict it.",
            )
        };
        let tasks_validate = Arc::new({ let s = state.clone(); move || T::validate_function(&s) });
        if tasks_validate().is_err() {
        errored = false;
        let mut step = run_step(
            agent_client.clone(), ctx.clone(), request.clone(),
            tasks_prompt, T::tasks_tools(&state),
            tasks_validate,
            id.clone(), created, object, continuation.take(), completion_index,
        );
        while let Some(output) = step.next().await {
            match output {
                StepOutput::Chunk(chunk) => {
                    errored = chunk.error.is_some();
                    for c in &chunk.completions {
                        if let Some(u) = &c.inner.usage {
                            accumulated_usage.push(u);
                        }
                    }
                    yield chunk;
                }
                StepOutput::Continuation(c) => { continuation = Some(c); }
                StepOutput::CompletionIndex(i) => { completion_index = i; }
            }
        }
        if errored {
            yield FunctionInventionChunk {
                id: id.to_string(), completions: vec![], state: None,
                path: None, function: None, created, object,
                usage: Some(accumulated_usage), error: None,
            };
            return;
        }
        }
        yield state_chunk(&state, &id, created, object);

        // Step 5: Description
        let description_validate = Arc::new({ let s = state.clone(); move || T::validate_description(&s) });
        if description_validate().is_err() {
        let description_prompt =
            "Create a 1-paragraph description of the Function you've invented. \
            The description should be concise (max 350 bytes) and summarize the \
            function's purpose. Read the Spec, Essay, and Tasks first.".to_string();
        errored = false;
        let mut step = run_step(
            agent_client.clone(), ctx.clone(), request.clone(),
            description_prompt, T::description_tools(&state),
            description_validate,
            id.clone(), created, object, continuation.take(), completion_index,
        );
        while let Some(output) = step.next().await {
            match output {
                StepOutput::Chunk(chunk) => {
                    errored = chunk.error.is_some();
                    for c in &chunk.completions {
                        if let Some(u) = &c.inner.usage {
                            accumulated_usage.push(u);
                        }
                    }
                    yield chunk;
                }
                StepOutput::Continuation(_) => {}
                StepOutput::CompletionIndex(_) => {}
            }
        }
        if errored {
            yield FunctionInventionChunk {
                id: id.to_string(), completions: vec![], state: None,
                path: None, function: None, created, object,
                usage: Some(accumulated_usage), error: None,
            };
            return;
        }
        }
        yield state_chunk(&state, &id, created, object);

        // Step 6: Readme (programmatic)
        T::write_readme(&state);

        // Final chunk: include the built function.
        let (final_state, function) = {
            let function = T::build_function(&state);
            let s = state.lock().unwrap().clone().into_state();
            (s, function)
        };

        // Clear filesystem before publishing if not persisting.
        if !persist {
            let _ = filesystem_client.clear();
        }

        // Publish the function if remote is set and build succeeded.
        let (path, publish_error) = if function.is_some() {
            if let Some(remote) = &request.remote {
                let publish_files = extract_publish_files(&final_state, function.as_ref().unwrap());
                let name = &T::params(&state).name;
                let description = extract_description(&final_state);
                match remote {
                    objectiveai::Remote::Filesystem => {
                        match publish_filesystem(
                            &filesystem_client, name, &publish_files,
                        ) {
                            Ok(path) => (Some(path), None),
                            Err(e) => (None, Some(e)),
                        }
                    }
                    objectiveai::Remote::Github => {
                        match publish_github(
                            &github_client, &filesystem_client,
                            &ctx, name, &description, &publish_files,
                        ).await {
                            Ok(path) => (Some(path), None),
                            Err(e) => (None, Some(e)),
                        }
                    }
                    objectiveai::Remote::Mock => (None, None),
                }
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        // Clear filesystem after publishing if not persisting.
        if !persist {
            let _ = filesystem_client.clear();
        }

        yield FunctionInventionChunk {
            id: id.to_string(),
            completions: vec![],
            state: Some(final_state),
            path,
            function,
            created,
            object,
            usage: Some(accumulated_usage),
            error: publish_error.map(|e| objectiveai::error::ResponseError {
                code: e.status(),
                message: e.message().unwrap_or(serde_json::Value::Null),
            }),
        };
    })
}

// ---------------------------------------------------------------------------
// Publishing helpers
// ---------------------------------------------------------------------------

/// Extracts the files to publish from the final invention state.
pub(crate) fn extract_publish_files(
    state: &objectiveai::functions::inventions::State,
    function: &objectiveai::functions::FullRemoteFunction,
) -> Vec<(&'static str, String)> {
    use objectiveai::functions::inventions::State;

    macro_rules! state_fields {
        ($s:expr) => {{
            let params_json = serde_json::to_string_pretty(&$s.params)
                .unwrap_or_default();
            let essay = $s.essay.clone().unwrap_or_default();
            let essay_tasks = $s.essay_tasks.clone().unwrap_or_default();
            let readme = $s.readme.clone().unwrap_or_default();
            (params_json, essay, essay_tasks, readme)
        }};
    }

    let (params_json, essay, essay_tasks, readme) = match state {
        State::AlphaScalarBranch(s) => state_fields!(s),
        State::AlphaScalarLeaf(s) => state_fields!(s),
        State::AlphaVectorBranch(s) => state_fields!(s),
        State::AlphaVectorLeaf(s) => state_fields!(s),
    };

    let function_json = serde_json::to_string_pretty(function)
        .unwrap_or_default();

    vec![
        ("function.json", function_json),
        ("parameters.json", params_json),
        ("ESSAY.md", essay),
        ("ESSAY_TASKS.md", essay_tasks),
        ("README.md", readme),
    ]
}

/// Extracts the description from the final invention state.
pub(crate) fn extract_description(state: &objectiveai::functions::inventions::State) -> String {
    use objectiveai::functions::inventions::State;
    match state {
        State::AlphaScalarBranch(s) => s.description.clone().unwrap_or_default(),
        State::AlphaScalarLeaf(s) => s.description.clone().unwrap_or_default(),
        State::AlphaVectorBranch(s) => s.description.clone().unwrap_or_default(),
        State::AlphaVectorLeaf(s) => s.description.clone().unwrap_or_default(),
    }
}

/// Publishes to the local filesystem.
///
/// `name` is `"owner/repository"`. Creates/resets the git repo, writes files,
/// and commits.
pub(crate) fn publish_filesystem(
    filesystem_client: &crate::filesystem::Client,
    name: &str,
    files: &[(&'static str, String)],
) -> Result<objectiveai::RemotePath, super::Error> {
    let (owner, repo) = name.split_once('/')
        .ok_or_else(|| super::Error::InvalidName(
            format!("name must be 'owner/repository', got '{}'", name),
        ))?;

    let file_refs: Vec<(&str, &str)> = files.iter()
        .map(|(n, c)| (*n, c.as_str()))
        .collect();

    let commit = filesystem_client
        .publish(crate::retrieval::Kind::Functions, owner, repo, &file_refs, &format!("publish {}", name))?;

    Ok(objectiveai::RemotePath::Filesystem {
        owner: owner.to_string(),
        repository: repo.to_string(),
        commit,
    })
}

/// Publishes to GitHub.
///
/// Delegates to [`crate::github::Client::publish`] which handles repo
/// creation, local git operations (via filesystem client), and push.
pub(crate) async fn publish_github<CTXEXT: ctx::ContextExt + Send + Sync>(
    github_client: &crate::github::Client,
    filesystem_client: &crate::filesystem::Client,
    ctx: &ctx::Context<CTXEXT>,
    name: &str,
    description: &str,
    files: &[(&'static str, String)],
) -> Result<objectiveai::RemotePath, super::Error> {
    let file_refs: Vec<(&str, &str)> = files.iter()
        .map(|(n, c)| (*n, c.as_str()))
        .collect();
    Ok(github_client
        .publish(filesystem_client, ctx, name, description, &file_refs)
        .await?)
}

// ---------------------------------------------------------------------------
// Single step runner — streams chunks as they arrive
// ---------------------------------------------------------------------------

/// Output from a single step.
enum StepOutput<OPENROUTER, CLAUDEAGENTSDK, MOCK>
where
    OPENROUTER: crate::agent::completions::UpstreamClient<objectiveai::agent::openrouter::Agent>,
    CLAUDEAGENTSDK: crate::agent::completions::UpstreamClient<objectiveai::agent::claude_agent_sdk::Agent>,
    MOCK: crate::agent::completions::UpstreamClient<objectiveai::agent::mock::Agent>,
{
    Chunk(FunctionInventionChunk),
    Continuation(Continuation<OPENROUTER, CLAUDEAGENTSDK, MOCK>),
    CompletionIndex(u64),
}

/// Builds `AgentCompletionCreateParams` from the invention request.
///
/// The `messages` field is only populated for the very first step (when no
/// continuation exists). For all subsequent steps and retries the prompt is
/// pushed as a user message onto the continuation so that the upstream sees
/// one continuous conversation.
fn build_agent_params(
    request: &objectiveai::functions::inventions::request::FunctionInventionCreateParams,
    messages: Vec<objectiveai::agent::completions::message::Message>,
) -> objectiveai::agent::completions::request::AgentCompletionCreateParams {
    objectiveai::agent::completions::request::AgentCompletionCreateParams {
        messages,
        provider: request.provider.clone(),
        agent: request.agent.clone(),
        response_format: None,
        seed: request.seed,
        stream: Some(true),
    }
}

/// Creates a user message from a prompt string.
fn user_message(prompt: &str) -> objectiveai::agent::completions::message::UserMessage {
    objectiveai::agent::completions::message::UserMessage {
        content: objectiveai::agent::completions::message::RichContent::Text(
            prompt.to_string(),
        ),
        name: None,
    }
}

fn run_step<CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, RETRG, RETRF, RETRM, CUSG>(
    agent_client: Arc<
        crate::agent::completions::Client<
            CTXEXT, OPENROUTER, CLAUDEAGENTSDK, MOCK, RETRG, RETRF, RETRM, CUSG,
        >,
    >,
    ctx: ctx::Context<CTXEXT>,
    request: Arc<objectiveai::functions::inventions::request::FunctionInventionCreateParams>,
    prompt: String,
    tools: Vec<objectiveai::functions::inventions::InventionTool>,
    validate: Arc<dyn Fn() -> Result<(), String> + Send + Sync>,
    id: String,
    created: u64,
    object: Object,
    initial_continuation: Option<Continuation<OPENROUTER, CLAUDEAGENTSDK, MOCK>>,
    initial_completion_index: u64,
) -> Pin<
    Box<
        dyn Stream<Item = StepOutput<OPENROUTER, CLAUDEAGENTSDK, MOCK>>
            + Send,
    >,
>
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
    RETRG: crate::retrieval::retrieve::Client<CTXEXT>,
    RETRF: crate::retrieval::retrieve::Client<CTXEXT>,
    RETRM: crate::retrieval::retrieve::Client<CTXEXT>,
    CUSG: crate::agent::completions::usage_handler::UsageHandler<CTXEXT> + Send + Sync + 'static,
{
    Box::pin(async_stream::stream! {
        let mut continuation = initial_continuation;
        let mut completion_index = initial_completion_index;
        let validate_for_done = validate.clone();
        let max_step_retries = request.max_step_retries.unwrap_or(3);

        // The messages array on the request is fixed after the very first
        // step. If we have a continuation (i.e. this is not the first step),
        // the prompt goes as a user message on the continuation and the
        // request messages are empty. If no continuation exists (first step),
        // the prompt goes into the request messages.
        let agent_params = if let Some(ref mut cont) = continuation {
            cont.push_user_message(user_message(&prompt));
            Arc::new(build_agent_params(&request, vec![]))
        } else {
            Arc::new(build_agent_params(
                &request,
                vec![objectiveai::agent::completions::message::Message::User(
                    user_message(&prompt),
                )],
            ))
        };

        // The agent completions loop handles tool calling internally via
        // invention_done. When validate returns Ok, invention_done fires,
        // tools_enabled becomes false, and the model produces a final
        // content-only response that ends the loop naturally.
        let invention_done = Arc::new(move || validate_for_done().is_ok());

        let stream_result = agent_client
            .create_streaming(
                ctx.clone(),
                agent_params.clone(),
                continuation.take(),
                Some(tools.clone()),
                Some(invention_done),
                None,
            )
            .await;

        let stream = match stream_result {
            Ok(stream) => stream,
            Err(e) => {
                yield StepOutput::Chunk(FunctionInventionChunk {
                    id: id.clone(),
                    completions: vec![],
                    state: None,
                    path: None,
                    function: None,
                    created,
                    object,
                    usage: None,
                    error: Some(objectiveai::error::ResponseError {
                        code: {
                            use objectiveai::error::StatusError;
                            e.status()
                        },
                        message: {
                            use objectiveai::error::StatusError;
                            e.message().unwrap_or(serde_json::Value::Null)
                        },
                    }),
                });
                return;
            }
        };

        futures::pin_mut!(stream);
        while let Some(item) = stream.next().await {
            match item {
                crate::agent::completions::StreamItem::Chunk(chunk) => {
                    yield StepOutput::Chunk(FunctionInventionChunk {
                        id: id.clone(),
                        completions: vec![InventionAgentCompletionChunk {
                            index: completion_index,
                            inner: chunk,
                        }],
                        state: None,
                        path: None,
                        function: None,
                        created,
                        object,
                        usage: None,
                        error: None,
                    });
                }
                crate::agent::completions::StreamItem::State(cont) => {
                    continuation = Some(cont);
                }
            }
        }

        // If validate still fails after the agent loop ended, start a new
        // agent completion with a retry prompt on the continuation. The
        // retry prompt includes the validation error, matching the pattern
        // from objectiveai-cli: prompt + "\n\n" + error + "\n\nPlease try again."
        let mut retries = 0u32;
        loop {
            let validation_error = match validate() {
                Ok(()) => break,
                Err(e) => e,
            };
            if retries >= max_step_retries {
                break;
            }
            retries += 1;

            let retry_prompt = format!(
                "{}\n\nThe following error occurred: {}\n\nPlease try again.",
                prompt, validation_error,
            );

            if let Some(ref mut cont) = continuation {
                cont.push_user_message(user_message(&retry_prompt));
            }

            completion_index += 1;

            let validate_for_done = validate.clone();
            let invention_done = Arc::new(move || validate_for_done().is_ok());

            let stream_result = agent_client
                .create_streaming(
                    ctx.clone(),
                    agent_params.clone(),
                    continuation.take(),
                    Some(tools.clone()),
                    Some(invention_done),
                    None,
                )
                .await;

            let stream = match stream_result {
                Ok(stream) => stream,
                Err(e) => {
                    yield StepOutput::Chunk(FunctionInventionChunk {
                        id: id.clone(),
                        completions: vec![],
                        state: None,
                        path: None,
                        function: None,
                        created,
                        object,
                        usage: None,
                        error: Some(objectiveai::error::ResponseError {
                            code: {
                                use objectiveai::error::StatusError;
                                e.status()
                            },
                            message: {
                                use objectiveai::error::StatusError;
                                e.message().unwrap_or(serde_json::Value::Null)
                            },
                        }),
                    });
                    return;
                }
            };

            futures::pin_mut!(stream);
            while let Some(item) = stream.next().await {
                match item {
                    crate::agent::completions::StreamItem::Chunk(chunk) => {
                        yield StepOutput::Chunk(FunctionInventionChunk {
                            id: id.clone(),
                            completions: vec![InventionAgentCompletionChunk {
                                index: completion_index,
                                inner: chunk,
                            }],
                            state: None,
                            path: None,
                            function: None,
                            created,
                            object,
                            usage: None,
                            error: None,
                        });
                    }
                    crate::agent::completions::StreamItem::State(cont) => {
                        continuation = Some(cont);
                    }
                }
            }
        }

        // Yield final continuation and completion index for the next step.
        if let Some(cont) = continuation {
            yield StepOutput::Continuation(cont);
        }
        yield StepOutput::CompletionIndex(completion_index + 1);
    })
}

/// Computes the task count string for prompts based on params.
fn tasks_str(params: &Params) -> String {
    let (min, max) = if params.depth > 0 {
        (params.min_branch_width, params.max_branch_width)
    } else {
        (params.min_leaf_width, params.max_leaf_width)
    };
    if min == max {
        format!("{min}")
    } else {
        format!("between {min} and {max}")
    }
}
