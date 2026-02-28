use crate::{ctx, util::StreamOnce};
use futures::{Stream, StreamExt};
use std::{
    pin::Pin,
    sync::{Arc, Mutex},
    time,
};

type Tool = objectiveai::functions::inventions::Tool;
type FunctionInventionChunk =
    objectiveai::functions::inventions::response::streaming::FunctionInventionChunk;
type Object =
    objectiveai::functions::inventions::response::streaming::Object;
type Params = objectiveai::functions::inventions::Params;
type State = objectiveai::functions::inventions::State;

use objectiveai::functions::inventions::InventionState;

/// Output from a single step — either a streamable chunk or the final upstream state.
enum StepOutput {
    Chunk(FunctionInventionChunk),
    UpstreamState(Option<serde_json::Value>),
}

/// Generates a unique response ID for Function inventions.
pub fn invention_response_id(created: u64) -> String {
    let uuid = uuid::Uuid::new_v4();
    format!("fncinv-{}-{}", uuid.simple(), created)
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/// Client for inventing Functions.
///
/// Orchestrates the multi-step invention flow: essay, input schema,
/// essay tasks, tasks, description, and readme generation.
pub struct Client<CTXEXT, IUSG> {
    pub upstream_client: super::upstream::Client,
    pub usage_handler: Arc<IUSG>,
    _ctxext: std::marker::PhantomData<CTXEXT>,
}

impl<CTXEXT, IUSG> Client<CTXEXT, IUSG> {
    pub fn new(
        upstream_client: super::upstream::Client,
        usage_handler: Arc<IUSG>,
    ) -> Self {
        Self {
            upstream_client,
            usage_handler,
            _ctxext: std::marker::PhantomData,
        }
    }
}

impl<CTXEXT, IUSG> Client<CTXEXT, IUSG>
where
    CTXEXT: Send + Sync + 'static,
    IUSG: super::usage_handler::UsageHandler<CTXEXT> + Send + Sync + 'static,
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
                    objectiveai::vector::completions::response::Usage::any_usage,
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
        _ctx: ctx::Context<CTXEXT>,
        request: Arc<
            objectiveai::functions::inventions::request::FunctionInventionCreateParams,
        >,
    ) -> Result<
        impl Stream<Item = FunctionInventionChunk> + Send + 'static,
        super::Error,
    > {
        let created = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let id = invention_response_id(created);
        let state = request.state.clone().route();
        let upstream_client = self.upstream_client.clone();

        let stream: Pin<Box<dyn Stream<Item = FunctionInventionChunk> + Send>> =
            match state {
                State::AlphaScalarBranch(s) => {
                    run_all_steps(s, upstream_client, request, id, created)
                }
                State::AlphaScalarLeaf(s) => {
                    run_all_steps(s, upstream_client, request, id, created)
                }
                State::AlphaVectorBranch(s) => {
                    run_all_steps(s, upstream_client, request, id, created)
                }
                State::AlphaVectorLeaf(s) => {
                    run_all_steps(s, upstream_client, request, id, created)
                }
            };

        Ok(stream)
    }
}

// ---------------------------------------------------------------------------
// Step orchestration
// ---------------------------------------------------------------------------

fn run_all_steps<T: InventionState>(
    state_val: T,
    upstream_client: super::upstream::Client,
    request: Arc<objectiveai::functions::inventions::request::FunctionInventionCreateParams>,
    id: String,
    created: u64,
) -> Pin<Box<dyn Stream<Item = FunctionInventionChunk> + Send>> {
    Box::pin(async_stream::stream! {
        let state = Arc::new(Mutex::new(state_val));
        let params = T::params(&state);
        let is_scalar = T::is_scalar();
        let object = T::object();
        let tasks_str = tasks_str(&params);

        // Step 1: Essay
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
        let mut upstream_state: Option<serde_json::Value> = None;
        let state_chunk = |state: &Arc<Mutex<T>>, id: &str, created, object| {
            FunctionInventionChunk {
                id: id.to_string(),
                completions: vec![],
                state: Some(state.lock().unwrap().clone().into_state()),
                function: None,
                created,
                object,
                usage: None,
                error: None,
            }
        };

        // Initial state
        yield state_chunk(&state, &id, created, object);

        // Step 1: Essay
        let mut step = run_step(
            upstream_client.clone(), request.clone(),
            essay_prompt, T::essay_tools(&state),
            Arc::new({ let s = state.clone(); move || T::validate_essay(&s) }),
            id.clone(), created, object, upstream_state,
        );
        let mut errored = false;
        upstream_state = None;
        while let Some(output) = step.next().await {
            match output {
                StepOutput::Chunk(chunk) => {
                    errored = chunk.error.is_some();
                    yield chunk;
                }
                StepOutput::UpstreamState(s) => { upstream_state = s; }
            }
        }
        if errored { return; }
        yield state_chunk(&state, &id, created, object);

        // Step 2: Input Schema
        let input_schema_prompt = if is_scalar {
            "Create the InputSchema for your Scalar Function. \
            Ensure that it adheres to the specifications outlined in your Spec \
            and is consistent with the essay you wrote describing your function. \
            Use CheckFunction after writing the input schema to validate it.".to_string()
        } else {
            "Create the InputSchema for your Vector Function. \
            Ensure that it adheres to the specifications outlined in your Spec \
            and is consistent with the essay you wrote describing your function. \
            Use CheckFunction after writing the input schema to validate it.".to_string()
        };
        let mut step = run_step(
            upstream_client.clone(), request.clone(),
            input_schema_prompt, T::input_schema_tools(&state),
            Arc::new({ let s = state.clone(); move || T::validate_input_schema(&s) }),
            id.clone(), created, object, upstream_state,
        );
        errored = false;
        upstream_state = None;
        while let Some(output) = step.next().await {
            match output {
                StepOutput::Chunk(chunk) => {
                    errored = chunk.error.is_some();
                    yield chunk;
                }
                StepOutput::UpstreamState(s) => { upstream_state = s; }
            }
        }
        if errored { return; }
        yield state_chunk(&state, &id, created, object);

        // Step 3: Essay Tasks
        let essay_tasks_prompt = format!(
            "Write EssayTasks listing and describing the key tasks the Function must \
            perform in order to fulfill the quality and value evaluations defined within \
            the essay. Each task is a non-technical plain language description of a task \
            which will go into the function's `tasks` array. There should be {tasks_str} tasks. \
            Read the Spec and Essay first.",
        );
        let mut step = run_step(
            upstream_client.clone(), request.clone(),
            essay_tasks_prompt, T::essay_tasks_tools(&state),
            Arc::new({ let s = state.clone(); move || T::validate_essay_tasks(&s) }),
            id.clone(), created, object, upstream_state,
        );
        errored = false;
        upstream_state = None;
        while let Some(output) = step.next().await {
            match output {
                StepOutput::Chunk(chunk) => {
                    errored = chunk.error.is_some();
                    yield chunk;
                }
                StepOutput::UpstreamState(s) => { upstream_state = s; }
            }
        }
        if errored { return; }
        yield state_chunk(&state, &id, created, object);

        // Step 4: Tasks (Body)
        let tasks_prompt = if is_scalar {
            if params.depth > 0 {
                format!(
                    "Create the Tasks for your Scalar Function. Create {tasks_str} placeholder tasks \
                    based on your EssayTasks. Each task defines a sub-function which will be \
                    automatically invented after you finish. \
                    For TaskSpec: write a detailed `spec` for the task describing what the \
                    sub-function should evaluate. \
                    Task Guidelines: `skip` (conditional skip expression), `input` (derives task \
                    input from parent input), `output` (transforms sub-function result). \
                    Expression context: `input` (function input), `output` (in output expressions). \
                    Use CheckFunction to validate. Re-read Spec first — it is the source of truth.",
                )
            } else {
                format!(
                    "Create the Tasks for your Scalar Function. Create {tasks_str} vector completion \
                    tasks based on your EssayTasks. Each task defines a prompt for an LLM \
                    as well as possible responses for the assistant to reply with. \
                    The ObjectiveAI system will return a vector of scores evaluating which \
                    response the LLM is most likely to reply with. These probabilities form \
                    the fundamental basis for how the Function scores the input. \
                    Be clever — do not ask the LLM to directly evaluate items. Make items \
                    into real responses. Each response should correspond to some score. \
                    Scores should be normalized so an equalized response vector yields a \
                    final score of 0.5. \
                    Expression context: `input` (function input), `output` (in output expressions). \
                    Use CheckFunction to validate. Re-read Spec first — it is the source of truth.",
                )
            }
        } else if params.depth > 0 {
            format!(
                "Create the Tasks for your Vector Function. Create {tasks_str} placeholder tasks \
                based on your EssayTasks. You can mix two types: \
                Unmapped vector tasks (placeholder.vector.function) rank the input items. \
                Mapped scalar tasks (placeholder.scalar.function with map) iterate over \
                input items and score each individually. At most 50% of tasks can be \
                mapped scalar tasks. \
                For TaskSpec: write a detailed `spec` for the task describing what the \
                sub-function should evaluate. \
                For Vector Tasks: create InputSchema, OutputLength, InputSplit, InputMerge expressions. \
                For Mapped Scalar Tasks: define an InputMap expression. \
                Task fields: `skip`, `input`, `output`. \
                Expression context: `input`, `map` (for mapped tasks), `output` (in output expressions). \
                Use CheckFunction to validate. Re-read Spec first — it is the source of truth.",
            )
        } else {
            format!(
                "Create the Tasks for your Vector Function. Create {tasks_str} vector completion \
                tasks based on your EssayTasks. Each task defines a prompt for an LLM \
                as well as possible responses for the assistant to reply with. \
                The ObjectiveAI system will return a vector of scores evaluating which \
                response the LLM is most likely to reply with. These probabilities form \
                the fundamental basis for how the Function ranks items. \
                Messages never contain the items to be ranked — items go into responses. \
                Be clever — do not ask the LLM to directly evaluate items. Make items \
                into real responses. \
                Expression context: `input` (function input), `output` (in output expressions). \
                Use CheckFunction to validate. Re-read Spec first — it is the source of truth.",
            )
        };
        let mut step = run_step(
            upstream_client.clone(), request.clone(),
            tasks_prompt, T::tasks_tools(&state),
            Arc::new({ let s = state.clone(); move || T::validate_function(&s) }),
            id.clone(), created, object, upstream_state,
        );
        errored = false;
        upstream_state = None;
        while let Some(output) = step.next().await {
            match output {
                StepOutput::Chunk(chunk) => {
                    errored = chunk.error.is_some();
                    yield chunk;
                }
                StepOutput::UpstreamState(s) => { upstream_state = s; }
            }
        }
        if errored { return; }
        yield state_chunk(&state, &id, created, object);

        // Step 5: Description
        let description_prompt =
            "Create a 1-paragraph description of the Function you've invented. \
            The description should be concise (max 350 bytes) and summarize the \
            function's purpose. Read the Spec, Essay, and Tasks first.".to_string();
        let mut step = run_step(
            upstream_client.clone(), request.clone(),
            description_prompt, T::description_tools(&state),
            Arc::new({ let s = state.clone(); move || T::validate_description(&s) }),
            id.clone(), created, object, upstream_state,
        );
        errored = false;
        while let Some(output) = step.next().await {
            match output {
                StepOutput::Chunk(chunk) => {
                    errored = chunk.error.is_some();
                    yield chunk;
                }
                StepOutput::UpstreamState(_) => {}
            }
        }
        if errored { return; }
        yield state_chunk(&state, &id, created, object);

        // Step 6: Readme (programmatic)
        T::write_readme(&state);
        yield state_chunk(&state, &id, created, object);
    })
}

// ---------------------------------------------------------------------------
// Single step runner — streams chunks as they arrive
// ---------------------------------------------------------------------------

fn run_step(
    upstream_client: super::upstream::Client,
    request: Arc<objectiveai::functions::inventions::request::FunctionInventionCreateParams>,
    prompt: String,
    tools: Vec<Tool>,
    validate: Arc<dyn Fn() -> Result<(), String> + Send + Sync>,
    id: String,
    created: u64,
    object: Object,
    initial_upstream_state: Option<serde_json::Value>,
) -> Pin<Box<dyn Stream<Item = StepOutput> + Send>> {
    Box::pin(async_stream::stream! {
        let mut upstream_state: Option<serde_json::Value> = initial_upstream_state;

        loop {
            let (stream, new_state) = match upstream_client
                .create_streaming(
                    request.clone(),
                    prompt.clone(),
                    tools.clone(),
                    upstream_state,
                )
                .await
            {
                Ok(result) => result,
                Err(e) => {
                    yield StepOutput::Chunk(FunctionInventionChunk {
                        id: id.clone(),
                        completions: vec![],
                        state: None,
                        function: None,
                        created,
                        object,
                        usage: None,
                        error: Some(objectiveai::error::ResponseError::from(&e)),
                    });
                    return;
                }
            };

            upstream_state = Some(new_state);

            futures::pin_mut!(stream);
            while let Some(completion_chunk) = stream.next().await {
                yield StepOutput::Chunk(FunctionInventionChunk {
                    id: id.clone(),
                    completions: vec![completion_chunk],
                    state: None,
                    function: None,
                    created,
                    object,
                    usage: None,
                    error: None,
                });
            }

            if validate().is_ok() {
                break;
            }
        }
        yield StepOutput::UpstreamState(upstream_state);
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
