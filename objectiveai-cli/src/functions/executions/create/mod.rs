use clap::{Args, Subcommand};
use futures::StreamExt;

/// How input is provided to the function execution.
#[derive(Args)]
#[group(required = true, multiple = false)]
pub struct InputSource {
    /// Inline JSON input value
    #[arg(long)]
    input_inline: Option<String>,
    /// Inline Python code that produces the input value
    #[arg(long)]
    input_python_inline: Option<String>,
    /// Path to a Python file that produces the input value
    #[arg(long)]
    input_python_file: Option<std::path::PathBuf>,
}

impl InputSource {
    fn resolve(self) -> Result<objectiveai::functions::expression::InputValue, crate::error::Error> {
        if let Some(inline) = self.input_inline {
            let mut de = serde_json::Deserializer::from_str(&inline);
            return serde_path_to_error::deserialize(&mut de)
                .map_err(crate::error::Error::PythonDeserialize);
        }
        if let Some(code) = self.input_python_inline {
            return crate::python::exec_code(&code);
        }
        if let Some(path) = self.input_python_file {
            return crate::python::exec_file(&path);
        }
        unreachable!("clap group ensures one is set")
    }
}

/// Where in the execution tree an error occurred.
pub enum ErrorPath {
    Root,
    Task(Vec<u64>),
    Reasoning,
}

impl serde::Serialize for ErrorPath {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ErrorPath::Root => serializer.serialize_str("root"),
            ErrorPath::Task(path) => path.serialize(serializer),
            ErrorPath::Reasoning => serializer.serialize_str("reasoning"),
        }
    }
}

/// A collected error with its location in the execution tree.
#[derive(serde::Serialize)]
pub struct CollectedError {
    pub path: ErrorPath,
    #[serde(flatten)]
    pub error: objectiveai::error::ResponseError,
}

/// The final result of a function execution.
#[derive(serde::Serialize)]
pub struct ExecutionResult {
    pub output: objectiveai::functions::expression::TaskOutputOwned,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<CollectedError>,
}

/// Recursively collect errors from the aggregated chunk.
fn collect_errors(chunk: &objectiveai::functions::executions::response::streaming::FunctionExecutionChunk, errors: &mut Vec<CollectedError>) {
    if let Some(err) = &chunk.error {
        errors.push(CollectedError {
            path: ErrorPath::Root,
            error: err.clone(),
        });
    }
    for task in &chunk.tasks {
        match task {
            objectiveai::functions::executions::response::streaming::TaskChunk::FunctionExecution(ft) => {
                if let Some(err) = &ft.inner.error {
                    errors.push(CollectedError {
                        path: ErrorPath::Task(ft.task_path.clone()),
                        error: err.clone(),
                    });
                }
                collect_errors(&ft.inner, errors);
            }
            objectiveai::functions::executions::response::streaming::TaskChunk::VectorCompletion(vt) => {
                if let Some(err) = &vt.error {
                    errors.push(CollectedError {
                        path: ErrorPath::Task(vt.task_path.clone()),
                        error: err.clone(),
                    });
                }
            }
        }
    }
    if let Some(reasoning) = &chunk.reasoning {
        if let Some(err) = &reasoning.error {
            errors.push(CollectedError {
                path: ErrorPath::Reasoning,
                error: err.clone(),
            });
        }
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Standard execution strategy (scalar or vector)
    Standard {
        /// Function reference (e.g. favorite=name or remote=github,owner=x,repository=y)
        #[arg(long)]
        function: crate::favorite_ref::FavoriteRef,
        /// Profile reference (e.g. favorite=name or remote=github,owner=x,repository=y)
        #[arg(long)]
        profile: crate::favorite_ref::FavoriteRef,
        #[command(flatten)]
        input: InputSource,
        #[command(flatten)]
        continuation: crate::continuation::ContinuationArgs,
        /// Retry token from a previous execution
        #[arg(long)]
        retry_token: Option<String>,
        /// Seed for deterministic mock responses
        #[arg(long)]
        seed: Option<i64>,
    },
    /// Swiss System tournament strategy (vector only)
    SwissSystem {
        /// Function reference (e.g. favorite=name or remote=github,owner=x,repository=y)
        #[arg(long)]
        function: crate::favorite_ref::FavoriteRef,
        /// Profile reference (e.g. favorite=name or remote=github,owner=x,repository=y)
        #[arg(long)]
        profile: crate::favorite_ref::FavoriteRef,
        #[command(flatten)]
        input: InputSource,
        #[command(flatten)]
        continuation: crate::continuation::ContinuationArgs,
        /// Retry token from a previous execution
        #[arg(long)]
        retry_token: Option<String>,
        /// Seed for deterministic mock responses
        #[arg(long)]
        seed: Option<i64>,
        /// How many vector responses per execution (default 10)
        #[arg(long)]
        pool: Option<usize>,
        /// How many sequential rounds of comparison (default 3)
        #[arg(long)]
        rounds: Option<usize>,
    },
}

async fn fn_favorites(cli_config: &crate::Config) -> Vec<objectiveai::filesystem::config::Favorite> {
    let (_, mut config) = crate::config::read(cli_config).await.unwrap();
    config.functions().get_favorites().to_vec()
}

async fn profile_favorites(cli_config: &crate::Config) -> Vec<objectiveai::filesystem::config::Favorite> {
    let (_, mut config) = crate::config::read(cli_config).await.unwrap();
    config.functions().profiles().get_favorites().to_vec()
}

impl Commands {
    pub async fn handle(self, cli_config: &crate::Config) -> Result<crate::Output, crate::error::Error> {
        let (function_path, profile_path, input_source, continuation_args, retry_token, seed, strategy) = match self {
            Commands::Standard { function, profile, input, continuation, retry_token, seed } => {
                let fp = function.resolve(|| fn_favorites(cli_config)).await?;
                let pp = profile.resolve(|| profile_favorites(cli_config)).await?;
                (fp, pp, input, continuation, retry_token, seed, objectiveai::functions::executions::request::Strategy::Default)
            }
            Commands::SwissSystem { function, profile, input, continuation, retry_token, seed, pool, rounds } => {
                let fp = function.resolve(|| fn_favorites(cli_config)).await?;
                let pp = profile.resolve(|| profile_favorites(cli_config)).await?;
                let strategy = objectiveai::functions::executions::request::Strategy::SwissSystem { pool, rounds };
                (fp, pp, input, continuation, retry_token, seed, strategy)
            }
        };

        let input_value = input_source.resolve()?;
        let continuation = continuation_args.resolve()?;

        let params = objectiveai::functions::executions::request::FunctionExecutionCreateParams {
            function: objectiveai::functions::FullInlineFunctionOrRemoteCommitOptional::Remote(function_path),
            profile: objectiveai::functions::InlineProfileOrRemoteCommitOptional::Remote(profile_path),
            retry_token,
            from_cache: None,
            reasoning: None,
            strategy: Some(strategy),
            input: input_value,
            provider: None,
            seed,
            stream: Some(true),
            continuation,
        };

        let log_writer = objectiveai::filesystem::logs::LogsClient::new(cli_config.config_base_dir.as_deref())
            .write_function_execution();

        crate::api::run(Box::new(|http_client| Box::pin(async move {
            let stream = objectiveai::functions::executions::create_function_execution_streaming(
                &http_client, params,
            ).await?;
            tokio::pin!(stream);

            // Aggregate all chunks into one
            let mut aggregated: Option<objectiveai::functions::executions::response::streaming::FunctionExecutionChunk> = None;
            let mut logged_path = false;
            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;
                match &mut aggregated {
                    Some(agg) => agg.push(&chunk),
                    None => aggregated = Some(chunk),
                }
                if let Some(agg) = &aggregated {
                    let _ = log_writer.write(agg).await;
                }
                if !logged_path {
                    if let Some(path) = log_writer.primary_path() {
                        eprintln!("In progress. Logs available at {path}.");
                        logged_path = true;
                    }
                }
            }

            let chunk = aggregated.ok_or(crate::error::Error::EmptyStream)?;

            // Recursively collect all errors
            let mut errors = Vec::new();
            collect_errors(&chunk, &mut errors);

            // Extract output (default to Err(null) if missing)
            let output = chunk.output
                .map(|o| o.unwrap())
                .unwrap_or(objectiveai::functions::expression::TaskOutputOwned::Err(serde_json::Value::Null));

            let result = ExecutionResult { output, errors };
            Ok(serde_json::to_string(&result).unwrap())
        })), true).await
    }
}
