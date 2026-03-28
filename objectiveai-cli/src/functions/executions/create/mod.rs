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

/// Function args for executions — supports mock remote via RemoteWithMock.
#[derive(Args)]
pub struct FunctionArgs {
    /// Get function by favorite name
    #[arg(long, conflicts_with_all = [
        "function_remote", "function_owner", "function_repository", "function_name", "function_commit"
    ])]
    pub function_favorite: Option<String>,
    /// Function remote source (github, filesystem, or mock)
    #[arg(long, value_enum,
        requires_if("github", "function_owner"),
        requires_if("github", "function_repository"),
        requires_if("filesystem", "function_owner"),
        requires_if("filesystem", "function_repository"),
        requires_if("mock", "function_name"),
    )]
    pub function_remote: Option<crate::remote::RemoteWithMock>,
    /// Function owner (github/filesystem)
    #[arg(long, conflicts_with = "function_name")]
    pub function_owner: Option<String>,
    /// Function repository (github/filesystem)
    #[arg(long, conflicts_with = "function_name")]
    pub function_repository: Option<String>,
    /// Function name (mock only)
    #[arg(long, conflicts_with_all = ["function_owner", "function_repository", "function_commit"])]
    pub function_name: Option<String>,
    /// Function commit (optional, github/filesystem only)
    #[arg(long)]
    pub function_commit: Option<String>,
}

impl FunctionArgs {
    fn resolve(self) -> Result<objectiveai::RemotePathCommitOptional, crate::error::Error> {
        if let Some(fav_name) = self.function_favorite {
            let (_, mut config) = crate::config::read()?;
            let favorites = config.functions().get_favorites().to_vec();
            let fav = favorites.into_iter().find(|f| f.get_name() == fav_name)
                .ok_or_else(|| crate::error::Error::FavoriteNotFound(fav_name))?;
            Ok(fav.path.clone())
        } else {
            self.function_remote
                .ok_or(crate::error::Error::MissingArgs("--function-remote is required (or use --function-favorite)"))?
                .into_path(self.function_owner, self.function_repository, self.function_name, self.function_commit)
                .ok_or(crate::error::Error::MissingArgs("--function-owner and --function-repository are required for github/filesystem, --function-name for mock"))
        }
    }
}

/// Profile args for executions — supports mock remote via RemoteWithMock.
#[derive(Args)]
pub struct ProfileArgs {
    /// Get profile by favorite name
    #[arg(long, conflicts_with_all = [
        "profile_remote", "profile_owner", "profile_repository", "profile_name", "profile_commit"
    ])]
    pub profile_favorite: Option<String>,
    /// Profile remote source (github, filesystem, or mock)
    #[arg(long, value_enum,
        requires_if("github", "profile_owner"),
        requires_if("github", "profile_repository"),
        requires_if("filesystem", "profile_owner"),
        requires_if("filesystem", "profile_repository"),
        requires_if("mock", "profile_name"),
    )]
    pub profile_remote: Option<crate::remote::RemoteWithMock>,
    /// Profile owner (github/filesystem)
    #[arg(long, conflicts_with = "profile_name")]
    pub profile_owner: Option<String>,
    /// Profile repository (github/filesystem)
    #[arg(long, conflicts_with = "profile_name")]
    pub profile_repository: Option<String>,
    /// Profile name (mock only)
    #[arg(long, conflicts_with_all = ["profile_owner", "profile_repository", "profile_commit"])]
    pub profile_name: Option<String>,
    /// Profile commit (optional, github/filesystem only)
    #[arg(long)]
    pub profile_commit: Option<String>,
}

impl ProfileArgs {
    fn resolve(self) -> Result<objectiveai::RemotePathCommitOptional, crate::error::Error> {
        if let Some(fav_name) = self.profile_favorite {
            let (_, mut config) = crate::config::read()?;
            let favorites = config.functions().profiles().get_favorites().to_vec();
            let fav = favorites.into_iter().find(|f| f.get_name() == fav_name)
                .ok_or_else(|| crate::error::Error::FavoriteNotFound(fav_name))?;
            Ok(fav.path.clone())
        } else {
            self.profile_remote
                .ok_or(crate::error::Error::MissingArgs("--profile-remote is required (or use --profile-favorite)"))?
                .into_path(self.profile_owner, self.profile_repository, self.profile_name, self.profile_commit)
                .ok_or(crate::error::Error::MissingArgs("--profile-owner and --profile-repository are required for github/filesystem, --profile-name for mock"))
        }
    }
}

#[derive(Subcommand)]
pub enum Commands {
    /// Standard execution strategy (scalar or vector)
    Standard {
        #[command(flatten)]
        function: FunctionArgs,
        #[command(flatten)]
        profile: ProfileArgs,
        #[command(flatten)]
        input: InputSource,
        /// Retry token from a previous execution
        #[arg(long)]
        retry_token: Option<String>,
        /// Seed for deterministic mock responses
        #[arg(long)]
        seed: Option<i64>,
    },
    /// Swiss System tournament strategy (vector only)
    SwissSystem {
        #[command(flatten)]
        function: FunctionArgs,
        #[command(flatten)]
        profile: ProfileArgs,
        #[command(flatten)]
        input: InputSource,
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

impl Commands {
    pub async fn handle(self) -> Result<crate::Output, crate::error::Error> {
        let (function_path, profile_path, input_source, retry_token, seed, strategy) = match self {
            Commands::Standard { function, profile, input, retry_token, seed } => {
                let fp = function.resolve()?;
                let pp = profile.resolve()?;
                (fp, pp, input, retry_token, seed, objectiveai::functions::executions::request::Strategy::Default)
            }
            Commands::SwissSystem { function, profile, input, retry_token, seed, pool, rounds } => {
                let fp = function.resolve()?;
                let pp = profile.resolve()?;
                let strategy = objectiveai::functions::executions::request::Strategy::SwissSystem { pool, rounds };
                (fp, pp, input, retry_token, seed, strategy)
            }
        };

        let input_value = input_source.resolve()?;

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
        };

        crate::api::run(|http_client| async move {
            let stream = objectiveai::functions::executions::create_function_execution_streaming(
                &http_client, params,
            ).await?;
            tokio::pin!(stream);

            // Aggregate all chunks into one
            let mut aggregated: Option<objectiveai::functions::executions::response::streaming::FunctionExecutionChunk> = None;
            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;
                match &mut aggregated {
                    Some(agg) => agg.push(&chunk),
                    None => aggregated = Some(chunk),
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
        }, true).await
    }
}
