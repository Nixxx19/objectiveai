//! ObjectiveAI API server.
//!
//! REST API server for chat completions, vector completions, Functions,
//! Profiles, Swarms, and authentication.

use axum::{
    Json,
    extract::Path,
    response::{IntoResponse, Sse, sse::Event},
};
use envconfig::Envconfig;
use objectiveai::error::ResponseError;
use objectiveai_api::{
    agent, auth, ctx, swarm,
    error::ResponseErrorExt,
    filesystem,
    functions::{self, profiles::computations::Client},
    github, mcp, objectiveai_http,
    util::StreamOnce,
    vector,
};
use std::{convert::Infallible, sync::Arc};
use tokio_stream::StreamExt;

#[derive(Envconfig)]
struct Config {
    #[envconfig(
        from = "OBJECTIVEAI_API_BASE",
        default = "https://api.objective-ai.io"
    )]
    objectiveai_api_base: String,
    #[envconfig(from = "OBJECTIVEAI_API_KEY")]
    objectiveai_api_key: Option<String>,
    #[envconfig(
        from = "OPENROUTER_API_BASE",
        default = "https://openrouter.ai/api/v1"
    )]
    openrouter_api_base: String,
    #[envconfig(from = "OPENROUTER_API_KEY")]
    openrouter_api_key: Option<String>,
    #[envconfig(from = "CLAUDE_AGENT_SDK", default = "0")]
    claude_agent_sdk: String,
    #[envconfig(from = "USER_AGENT")]
    user_agent: Option<String>,
    #[envconfig(from = "HTTP_REFERER")]
    http_referer: Option<String>,
    #[envconfig(from = "X_TITLE")]
    x_title: Option<String>,
    #[envconfig(
        from = "AGENT_COMPLETIONS_BACKOFF_CURRENT_INTERVAL",
        default = "100" // 100 milliseconds
    )]
    agent_completions_backoff_current_interval: u64,
    #[envconfig(
        from = "AGENT_COMPLETIONS_BACKOFF_INITIAL_INTERVAL",
        default = "100" // 100 milliseconds
    )]
    agent_completions_backoff_initial_interval: u64,
    #[envconfig(
        from = "AGENT_COMPLETIONS_BACKOFF_RANDOMIZATION_FACTOR",
        default = "0.5"
    )]
    agent_completions_backoff_randomization_factor: f64,
    #[envconfig(from = "AGENT_COMPLETIONS_BACKOFF_MULTIPLIER", default = "1.5")]
    agent_completions_backoff_multiplier: f64,
    #[envconfig(
        from = "AGENT_COMPLETIONS_BACKOFF_MAX_INTERVAL",
        default = "1000" // 1 second
    )]
    agent_completions_backoff_max_interval: u64,
    #[envconfig(
        from = "AGENT_COMPLETIONS_BACKOFF_MAX_ELAPSED_TIME",
        default = "40000" // 40 seconds
    )]
    agent_completions_backoff_max_elapsed_time: u64,
    #[envconfig(
        from = "MCP_BACKOFF_CURRENT_INTERVAL",
        default = "100" // 100 milliseconds
    )]
    mcp_backoff_current_interval: u64,
    #[envconfig(
        from = "MCP_BACKOFF_INITIAL_INTERVAL",
        default = "100" // 100 milliseconds
    )]
    mcp_backoff_initial_interval: u64,
    #[envconfig(
        from = "MCP_BACKOFF_RANDOMIZATION_FACTOR",
        default = "0.5"
    )]
    mcp_backoff_randomization_factor: f64,
    #[envconfig(from = "MCP_BACKOFF_MULTIPLIER", default = "1.5")]
    mcp_backoff_multiplier: f64,
    #[envconfig(
        from = "MCP_BACKOFF_MAX_INTERVAL",
        default = "1000" // 1 second
    )]
    mcp_backoff_max_interval: u64,
    #[envconfig(
        from = "MCP_BACKOFF_MAX_ELAPSED_TIME",
        default = "40000" // 40 seconds
    )]
    mcp_backoff_max_elapsed_time: u64,
    #[envconfig(
        from = "GITHUB_BACKOFF_CURRENT_INTERVAL",
        default = "100" // 100 milliseconds
    )]
    github_backoff_current_interval: u64,
    #[envconfig(
        from = "GITHUB_BACKOFF_INITIAL_INTERVAL",
        default = "100" // 100 milliseconds
    )]
    github_backoff_initial_interval: u64,
    #[envconfig(
        from = "GITHUB_BACKOFF_RANDOMIZATION_FACTOR",
        default = "0.5"
    )]
    github_backoff_randomization_factor: f64,
    #[envconfig(from = "GITHUB_BACKOFF_MULTIPLIER", default = "1.5")]
    github_backoff_multiplier: f64,
    #[envconfig(
        from = "GITHUB_BACKOFF_MAX_INTERVAL",
        default = "1000" // 1 second
    )]
    github_backoff_max_interval: u64,
    #[envconfig(
        from = "GITHUB_BACKOFF_MAX_ELAPSED_TIME",
        default = "40000" // 40 seconds
    )]
    github_backoff_max_elapsed_time: u64,
    #[envconfig(
        from = "AGENT_COMPLETIONS_FIRST_CHUNK_TIMEOUT",
        default = "60000" // 60 seconds
    )]
    agent_completions_first_chunk_timeout: u64,
    #[envconfig(
        from = "AGENT_COMPLETIONS_OTHER_CHUNK_TIMEOUT",
        default = "30000" // 30 seconds
    )]
    agent_completions_other_chunk_timeout: u64,
    #[envconfig(
        from = "MCP_CONNECT_TIMEOUT",
        default = "30000" // 30 seconds
    )]
    mcp_connect_timeout: u64,
    #[envconfig(
        from = "MCP_CALL_TIMEOUT",
        default = "30000" // 30 seconds
    )]
    mcp_call_timeout: u64,
    #[envconfig(from = "FETCH_GITHUB_TOKEN")]
    fetch_github_token: Option<String>,
    #[envconfig(from = "PUBLISH_GITHUB_TOKEN")]
    publish_github_token: Option<String>,
    #[envconfig(from = "FILESYSTEM_COMMIT_AUTHOR_NAME", default = "ObjectiveAI")]
    filesystem_commit_author_name: String,
    #[envconfig(from = "FILESYSTEM_COMMIT_AUTHOR_EMAIL", default = "admin@objective-ai.io")]
    filesystem_commit_author_email: String,
    #[envconfig(from = "MOCK_DELAY_MS", default = "0")]
    mock_delay_ms: u64,
    #[envconfig(from = "MOCK_MAX_TOOL_CALLS", default = "1000")]
    mock_max_tool_calls: u32,
    #[envconfig(from = "ADDRESS", default = "0.0.0.0")]
    address: String,
    #[envconfig(from = "PORT", default = "5000")]
    port: u16,
}

#[tokio::main]
async fn main() {
    // Load .env file if present
    let _ = dotenv::dotenv();

    // Load config from environment
    let Config {
        objectiveai_api_base,
        objectiveai_api_key,
        openrouter_api_base,
        openrouter_api_key,
        claude_agent_sdk,
        user_agent,
        http_referer,
        x_title,
        agent_completions_backoff_current_interval,
        agent_completions_backoff_initial_interval,
        agent_completions_backoff_randomization_factor,
        agent_completions_backoff_multiplier,
        agent_completions_backoff_max_interval,
        agent_completions_backoff_max_elapsed_time,
        mcp_backoff_current_interval,
        mcp_backoff_initial_interval,
        mcp_backoff_randomization_factor,
        mcp_backoff_multiplier,
        mcp_backoff_max_interval,
        mcp_backoff_max_elapsed_time,
        github_backoff_current_interval,
        github_backoff_initial_interval,
        github_backoff_randomization_factor,
        github_backoff_multiplier,
        github_backoff_max_interval,
        github_backoff_max_elapsed_time,
        agent_completions_first_chunk_timeout,
        agent_completions_other_chunk_timeout,
        mcp_connect_timeout,
        mcp_call_timeout,
        fetch_github_token,
        publish_github_token,
        filesystem_commit_author_name,
        filesystem_commit_author_email,
        mock_delay_ms,
        mock_max_tool_calls,
        address,
        port,
    } = Config::init_from_env().unwrap();

    // HTTP Client
    let http_client = reqwest::Client::new();

    // ObjectiveAI HTTP Client
    let objectiveai_http_client = Arc::new(objectiveai_http::Client::new(
        http_client.clone(),
        Some(objectiveai_api_base),
        objectiveai_api_key,
        user_agent.clone(),
        x_title.clone(),
        http_referer.clone(),
    ));

    // Swarm Fetcher
    let swarm_fetcher = Arc::new(swarm::fetcher::CachingFetcher::new(
        Arc::new(swarm::fetcher::ObjectiveAiFetcher::new(
            objectiveai_http_client.clone(),
        )),
    ));

    // Vector Completion Votes Fetcher
    let completion_votes_fetcher = Arc::new(
        vector::completions::completion_votes_fetcher::ObjectiveAiFetcher::new(
            objectiveai_http_client.clone(),
        ),
    );

    // Vector Cache Vote Fetcher
    let cache_vote_fetcher = Arc::new(
        vector::completions::cache_vote_fetcher::ObjectiveAiFetcher::new(
            objectiveai_http_client.clone(),
        ),
    );

    // GitHub Client
    let github_client = Arc::new(github::Client::new(
        reqwest::Client::new(),
        fetch_github_token,
        publish_github_token,
        user_agent,
        x_title,
        http_referer,
        std::time::Duration::from_millis(github_backoff_current_interval),
        std::time::Duration::from_millis(github_backoff_initial_interval),
        github_backoff_randomization_factor,
        github_backoff_multiplier,
        std::time::Duration::from_millis(github_backoff_max_interval),
        std::time::Duration::from_millis(github_backoff_max_elapsed_time),
    ));

    // Filesystem base directory for local function/profile repositories
    let filesystem_base_dir = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".objectiveai")
        .join("functions");

    let filesystem_client = Arc::new(filesystem::Client::new(
        filesystem_base_dir,
        filesystem_commit_author_name,
        filesystem_commit_author_email,
    ));

    // Function Fetcher (routes to GitHub, Filesystem, or Mock based on Remote)
    let function_fetcher =
        Arc::new(functions::function_fetcher::FetcherRouter::new(
            Arc::new(functions::function_fetcher::github::GithubFetcher::new(
                github_client.clone(),
            )),
            Arc::new(
                functions::function_fetcher::filesystem::FilesystemFetcher::new(
                    filesystem_client.clone(),
                ),
            ),
            Arc::new(functions::function_fetcher::mock::MockFetcher),
        ));

    // Function Profile Fetcher (routes to GitHub, Filesystem, or Mock based on Remote)
    let profile_fetcher =
        Arc::new(functions::profile_fetcher::FetcherRouter::new(
            Arc::new(functions::profile_fetcher::github::GithubFetcher::new(
                github_client.clone(),
            )),
            Arc::new(
                functions::profile_fetcher::filesystem::FilesystemFetcher::new(
                    filesystem_client.clone(),
                ),
            ),
            Arc::new(functions::profile_fetcher::mock::MockFetcher),
        ));

    // MCP Client
    let mcp_client = Arc::new(mcp::Client::new(
        reqwest::Client::new(),
        None, // user_agent already moved
        None, // x_title already moved
        None, // referer already moved
        std::time::Duration::from_millis(mcp_connect_timeout),
        std::time::Duration::from_millis(
            mcp_backoff_current_interval,
        ),
        std::time::Duration::from_millis(
            mcp_backoff_initial_interval,
        ),
        mcp_backoff_randomization_factor,
        mcp_backoff_multiplier,
        std::time::Duration::from_millis(mcp_backoff_max_interval),
        std::time::Duration::from_millis(
            mcp_backoff_max_elapsed_time,
        ),
        std::time::Duration::from_millis(mcp_call_timeout),
    ));

    // Agent Fetcher
    let agent_fetcher = Arc::new(agent::fetcher::CachingFetcher::new(
        Arc::new(agent::fetcher::ObjectiveAiFetcher::new(
            objectiveai_http_client.clone(),
        )),
    ));

    // Agent Completions Client
    let agent_completions_client = Arc::new(agent::completions::Client::new(
        mcp_client.clone(),
        agent_fetcher.clone(),
        Arc::new(agent::completions::usage_handler::LogUsageHandler),
        Arc::new(agent::completions::openrouter::Client {
            http_client: reqwest::Client::new(),
            api_base: openrouter_api_base.clone(),
            api_key: openrouter_api_key.clone().unwrap_or_default(),
            user_agent: None,
            x_title: None,
            referer: None,
        }),
        Arc::new(agent::completions::claude_agent_sdk::Client::new(None)),
        Arc::new(agent::completions::mock::Client {
            delay: std::time::Duration::from_millis(mock_delay_ms),
            max_tool_calls: mock_max_tool_calls,
        }),
        std::time::Duration::from_millis(
            agent_completions_backoff_current_interval,
        ),
        std::time::Duration::from_millis(
            agent_completions_backoff_initial_interval,
        ),
        agent_completions_backoff_randomization_factor,
        agent_completions_backoff_multiplier,
        std::time::Duration::from_millis(agent_completions_backoff_max_interval),
        std::time::Duration::from_millis(
            agent_completions_backoff_max_elapsed_time,
        ),
        std::time::Duration::from_millis(agent_completions_first_chunk_timeout),
        std::time::Duration::from_millis(agent_completions_other_chunk_timeout),
    ));

    // Vector Completions Client
    let vector_completions_client = Arc::new(vector::completions::Client::new(
        agent_completions_client.clone(),
        swarm_fetcher.clone(),
        completion_votes_fetcher.clone(),
        cache_vote_fetcher.clone(),
        Arc::new(vector::completions::usage_handler::LogUsageHandler),
    ));

    // Vector Completions Cache Client
    let vector_completions_cache_client =
        Arc::new(vector::completions::cache::Client::new(
            completion_votes_fetcher.clone(),
            cache_vote_fetcher.clone(),
        ));

    // Function Inventions Client
    let function_inventions_client =
        Arc::new(functions::inventions::Client::new(
            agent_completions_client.clone(),
            github_client.clone(),
            filesystem_client.clone(),
            function_fetcher.clone(),
            Arc::new(functions::inventions::usage_handler::LogUsageHandler),
            true, // persist
        ));

    // Function Inventions Recursive Client
    let function_inventions_recursive_client =
        Arc::new(functions::inventions::recursive::Client::new(
            function_inventions_client.clone(),
            Arc::new(
                functions::inventions::recursive::usage_handler::LogUsageHandler,
            ),
        ));

    // Function Executions Client
    let function_executions_client =
        Arc::new(functions::executions::Client::new(
            agent_completions_client.clone(),
            swarm_fetcher.clone(),
            vector_completions_client.clone(),
            function_fetcher.clone(),
            profile_fetcher.clone(),
            Arc::new(functions::executions::usage_handler::LogUsageHandler),
        ));

    // Functions Profiles Computations Client
    let profile_computations_client =
        Arc::new(functions::profiles::computations::ObjectiveAiClient::new(
            objectiveai_http_client.clone(),
        ));

    // Functions Client
    let functions_client = Arc::new(functions::Client::new(
        function_fetcher.clone(),
        Arc::new(functions::retrieval_client::ObjectiveAiClient::new(
            objectiveai_http_client.clone(),
        )),
    ));

    // Function Profiles Client
    let profiles_client = Arc::new(functions::profiles::Client::new(
        profile_fetcher.clone(),
        Arc::new(
            functions::profiles::retrieval_client::ObjectiveAiClient::new(
                objectiveai_http_client.clone(),
            ),
        ),
    ));

    // Function-Profile Pairs Client
    let pairs_client =
        Arc::new(functions::pair_retrieval_client::ObjectiveAiClient::new(
            objectiveai_http_client.clone(),
        ));

    // Auth Client
    let auth_client = Arc::new(auth::ObjectiveAiClient::new(
        objectiveai_http_client.clone(),
    ));

    // Swarm Client
    let swarm_client = Arc::new(swarm::Client::new(
        swarm_fetcher.clone(),
        Arc::new(swarm::retrieval_client::ObjectiveAiClient::new(
            objectiveai_http_client.clone(),
        )),
    ));

    // Agent Client (browse/list/get)
    let agent_client = Arc::new(agent::Client::new(
        agent_fetcher.clone(),
        Arc::new(agent::retrieval_client::ObjectiveAiClient::new(
            objectiveai_http_client.clone(),
        )),
    ));

    // Router
    let app = axum::Router::new()
        // Agent Completions - create
        .route(
            "/agent/completions",
            axum::routing::post({
                let agent_completions_client = agent_completions_client.clone();
                move |headers: axum::http::HeaderMap, Json(body): Json<
                    objectiveai::agent::completions::request::AgentCompletionCreateParams,
                >| {
                    create_agent_completion(agent_completions_client, headers, body)
                }
            }),
        )
        // Vector Completions - create
        .route(
            "/vector/completions",
            axum::routing::post({
                let vector_completions_client = vector_completions_client.clone();
                move |headers: axum::http::HeaderMap, Json(body): Json<
                    objectiveai::vector::completions::request::VectorCompletionCreateParams,
                >| {
                    create_vector_completion(vector_completions_client, headers, body)
                }
            }),
        )
        // Vector Completions - get completion votes
        .route(
            "/vector/completions/{id}",
            axum::routing::post({
                let vector_completions_cache_client =
                    vector_completions_cache_client.clone();
                move |Path(id): Path<String>, headers: axum::http::HeaderMap| {
                    get_vector_completion_votes(
                        vector_completions_cache_client,
                        headers,
                        id,
                    )
                }
            }),
        )
        // Vector Completions - get cache vote
        .route(
            "/vector/completions/cache",
            axum::routing::post({
                let vector_completions_cache_client =
                    vector_completions_cache_client.clone();
                move |headers: axum::http::HeaderMap, Json(body): Json<
                    objectiveai::vector::completions::cache::request::CacheVoteRequestOwned,
                >| {
                    get_vector_cache_vote(
                        vector_completions_cache_client,
                        headers,
                        body,
                    )
                }
            }),
        )
        // Functions - list
        .route(
            "/functions",
            axum::routing::get({
                let functions_client = functions_client.clone();
                move |headers: axum::http::HeaderMap| list_functions(functions_client, headers)
            }),
        )
        // Functions - get (without commit)
        .route(
            "/functions/{fremote}/{fowner}/{frepository}",
            axum::routing::get({
                let functions_client = functions_client.clone();
                move |Path((fremote, fowner, frepository)): Path<(objectiveai::functions::Remote, String, String)>, headers: axum::http::HeaderMap| {
                    get_function(functions_client, headers, fremote, fowner, frepository, None)
                }
            }),
        )
        // Functions - get (with commit)
        .route(
            "/functions/{fremote}/{fowner}/{frepository}/{fcommit}",
            axum::routing::get({
                let functions_client = functions_client.clone();
                move |Path((fremote, fowner, frepository, fcommit)): Path<(
                    objectiveai::functions::Remote,
                    String,
                    String,
                    String,
                )>, headers: axum::http::HeaderMap| {
                    get_function(
                        functions_client,
                        headers,
                        fremote,
                        fowner,
                        frepository,
                        Some(fcommit),
                    )
                }
            }),
        )
        // Functions - get usage (without commit)
        .route(
            "/functions/{fremote}/{fowner}/{frepository}/usage",
            axum::routing::get({
                let functions_client = functions_client.clone();
                move |Path((fremote, fowner, frepository)): Path<(objectiveai::functions::Remote, String, String)>, headers: axum::http::HeaderMap| {
                    get_function_usage(functions_client, headers, fremote, fowner, frepository, None)
                }
            }),
        )
        // Functions - get usage (with commit)
        .route(
            "/functions/{fremote}/{fowner}/{frepository}/{fcommit}/usage",
            axum::routing::get({
                let functions_client = functions_client.clone();
                move |Path((fremote, fowner, frepository, fcommit)): Path<(objectiveai::functions::Remote, String, String, String)>, headers: axum::http::HeaderMap| {
                    get_function_usage(
                        functions_client,
                        headers,
                        fremote,
                        fowner,
                        frepository,
                        Some(fcommit),
                    )
                }
            }),
        )
        // Function Executions - create
        // inline function
        // inline profile
        .route(
            "/functions",
            axum::routing::post({
                let function_executions_client = function_executions_client.clone();
                move |headers: axum::http::HeaderMap, Json(body): Json<
                    objectiveai::functions::executions::request::FunctionInlineProfileInlineRequestBody,
                >| {
                    execute_function(
                        function_executions_client,
                        headers,
                        objectiveai::functions::executions::request::Request::FunctionInlineProfileInline {
                            body,
                        },
                    )
                }
            }),
        )
        // Function Executions - create
        // remote function (without commit)
        // inline profile
        .route(
            "/functions/{fremote}/{fowner}/{frepository}",
            axum::routing::post({
                let function_executions_client = function_executions_client.clone();
                move |Path(path): Path<
                    objectiveai::functions::executions::request::FunctionRemoteProfileInlineRequestPath,
                >,
                      headers: axum::http::HeaderMap,
                      Json(body): Json<
                    objectiveai::functions::executions::request::FunctionRemoteProfileInlineRequestBody,
                >| {
                    execute_function(
                        function_executions_client,
                        headers,
                        objectiveai::functions::executions::request::Request::FunctionRemoteProfileInline {
                            path,
                            body,
                        },
                    )
                }
            }),
        )
        // Function Executions - create
        // remote function (with commit)
        // inline profile
        .route(
            "/functions/{fremote}/{fowner}/{frepository}/{fcommit}",
            axum::routing::post({
                let function_executions_client = function_executions_client.clone();
                move |Path(path): Path<
                    objectiveai::functions::executions::request::FunctionRemoteProfileInlineRequestPath,
                >,
                      headers: axum::http::HeaderMap,
                      Json(body): Json<
                    objectiveai::functions::executions::request::FunctionRemoteProfileInlineRequestBody,
                >| {
                    execute_function(
                        function_executions_client,
                        headers,
                        objectiveai::functions::executions::request::Request::FunctionRemoteProfileInline {
                            path,
                            body,
                        },
                    )
                }
            }),
        )
        // Function Executions - create
        // inline function
        // remote profile (without commit)
        .route(
            "/functions/profiles/{premote}/{powner}/{prepository}",
            axum::routing::post({
                let function_executions_client = function_executions_client.clone();
                move |Path(path): Path<
                    objectiveai::functions::executions::request::FunctionInlineProfileRemoteRequestPath,
                >,
                      headers: axum::http::HeaderMap,
                      Json(body): Json<
                    objectiveai::functions::executions::request::FunctionInlineProfileRemoteRequestBody,
                >| {
                    execute_function(
                        function_executions_client,
                        headers,
                        objectiveai::functions::executions::request::Request::FunctionInlineProfileRemote {
                            path,
                            body,
                        },
                    )
                }
            }),
        )
        // Function Executions - create
        // inline function
        // remote profile (with commit)
        .route(
            "/functions/profiles/{premote}/{powner}/{prepository}/{pcommit}",
            axum::routing::post({
                let function_executions_client = function_executions_client.clone();
                move |Path(path): Path<
                    objectiveai::functions::executions::request::FunctionInlineProfileRemoteRequestPath,
                >,
                      headers: axum::http::HeaderMap,
                      Json(body): Json<
                    objectiveai::functions::executions::request::FunctionInlineProfileRemoteRequestBody,
                >| {
                    execute_function(
                        function_executions_client,
                        headers,
                        objectiveai::functions::executions::request::Request::FunctionInlineProfileRemote {
                            path,
                            body,
                        },
                    )
                }
            }),
        )
        // Function Executions - create
        // remote function (without commit)
        // remote profile (without commit)
        .route(
            "/functions/{fremote}/{fowner}/{frepository}/profiles/{premote}/{powner}/{prepository}",
            axum::routing::post({
                let function_executions_client = function_executions_client.clone();
                move |Path(path): Path<
                    objectiveai::functions::executions::request::FunctionRemoteProfileRemoteRequestPath,
                >,
                      headers: axum::http::HeaderMap,
                      Json(body): Json<
                    objectiveai::functions::executions::request::FunctionRemoteProfileRemoteRequestBody,
                >| {
                    execute_function(
                        function_executions_client,
                        headers,
                        objectiveai::functions::executions::request::Request::FunctionRemoteProfileRemote {
                            path,
                            body,
                        },
                    )
                }
            }),
        )
        // Function Executions - create
        // remote function (without commit)
        // remote profile (with commit)
        .route(
            "/functions/{fremote}/{fowner}/{frepository}/profiles/{premote}/{powner}/{prepository}/{pcommit}",
            axum::routing::post({
                let function_executions_client = function_executions_client.clone();
                move |Path(path): Path<
                    objectiveai::functions::executions::request::FunctionRemoteProfileRemoteRequestPath,
                >,
                      headers: axum::http::HeaderMap,
                      Json(body): Json<
                    objectiveai::functions::executions::request::FunctionRemoteProfileRemoteRequestBody,
                >| {
                    execute_function(
                        function_executions_client,
                        headers,
                        objectiveai::functions::executions::request::Request::FunctionRemoteProfileRemote {
                            path,
                            body,
                        },
                    )
                }
            }),
        )
        // Function Executions - create
        // remote function (with commit)
        // remote profile (without commit)
        .route(
            "/functions/{fremote}/{fowner}/{frepository}/{fcommit}/profiles/{premote}/{powner}/{prepository}",
            axum::routing::post({
                let function_executions_client = function_executions_client.clone();
                move |Path(path): Path<
                    objectiveai::functions::executions::request::FunctionRemoteProfileRemoteRequestPath,
                >,
                      headers: axum::http::HeaderMap,
                      Json(body): Json<
                    objectiveai::functions::executions::request::FunctionRemoteProfileRemoteRequestBody,
                >| {
                    execute_function(
                        function_executions_client,
                        headers,
                        objectiveai::functions::executions::request::Request::FunctionRemoteProfileRemote {
                            path,
                            body,
                        },
                    )
                }
            }),
        )
        // Function Executions - create
        // remote function (with commit)
        // remote profile (with commit)
        .route(
            "/functions/{fremote}/{fowner}/{frepository}/{fcommit}/profiles/{premote}/{powner}/{prepository}/{pcommit}",
            axum::routing::post({
                let function_executions_client = function_executions_client.clone();
                move |Path(path): Path<
                    objectiveai::functions::executions::request::FunctionRemoteProfileRemoteRequestPath,
                >,
                      headers: axum::http::HeaderMap,
                      Json(body): Json<
                    objectiveai::functions::executions::request::FunctionRemoteProfileRemoteRequestBody,
                >| {
                    execute_function(
                        function_executions_client,
                        headers,
                        objectiveai::functions::executions::request::Request::FunctionRemoteProfileRemote {
                            path,
                            body,
                        },
                    )
                }
            }),
        )
        // Function Profiles - list
        .route(
            "/functions/profiles",
            axum::routing::get({
                let profiles_client = profiles_client.clone();
                move |headers: axum::http::HeaderMap| list_profiles(profiles_client, headers)
            }),
        )
        // Function Profiles - get (without commit)
        .route(
            "/functions/profiles/{premote}/{powner}/{prepository}",
            axum::routing::get({
                let profiles_client = profiles_client.clone();
                move |Path((premote, powner, prepository)): Path<(objectiveai::functions::Remote, String, String)>, headers: axum::http::HeaderMap| {
                    get_profile(profiles_client, headers, premote, powner, prepository, None)
                }
            }),
        )
        // Function Profiles - get (with commit)
        .route(
            "/functions/profiles/{premote}/{powner}/{prepository}/{pcommit}",
            axum::routing::get({
                let profiles_client = profiles_client.clone();
                move |Path((premote, powner, prepository, pcommit)): Path<(
                    objectiveai::functions::Remote,
                    String,
                    String,
                    String,
                )>, headers: axum::http::HeaderMap| {
                    get_profile(
                        profiles_client,
                        headers,
                        premote,
                        powner,
                        prepository,
                        Some(pcommit),
                    )
                }
            }),
        )
        // Function Profiles - get usage (without commit)
        .route(
            "/functions/profiles/{premote}/{powner}/{prepository}/usage",
            axum::routing::get({
                let profiles_client = profiles_client.clone();
                move |Path((premote, powner, prepository)): Path<(objectiveai::functions::Remote, String, String)>, headers: axum::http::HeaderMap| {
                    get_profile_usage(profiles_client, headers, premote, powner, prepository, None)
                }
            }),
        )
        // Function Profiles - get usage (with commit)
        .route(
            "/functions/profiles/{premote}/{powner}/{prepository}/{pcommit}/usage",
            axum::routing::get({
                let profiles_client = profiles_client.clone();
                move |Path((premote, powner, prepository, pcommit)): Path<(objectiveai::functions::Remote, String, String, String)>, headers: axum::http::HeaderMap| {
                    get_profile_usage(
                        profiles_client,
                        headers,
                        premote,
                        powner,
                        prepository,
                        Some(pcommit),
                    )
                }
            }),
        )
        // Function-Profile Pairs - list
        .route(
            "/functions/profiles/pairs",
            axum::routing::get({
                let pairs_client = pairs_client.clone();
                move |headers: axum::http::HeaderMap| list_function_profile_pairs(pairs_client, headers)
            }),
        )
        // Function-Profile Pairs - get usage (no commits)
        .route(
            "/functions/{fremote}/{fowner}/{frepository}/profiles/{premote}/{powner}/{prepository}/usage",
            axum::routing::get({
                let pairs_client = pairs_client.clone();
                move |Path((fremote, fowner, frepository, premote, powner, prepository)): Path<(objectiveai::functions::Remote, String, String, objectiveai::functions::Remote, String, String)>, headers: axum::http::HeaderMap| {
                    get_function_profile_pair_usage(
                        pairs_client,
                        headers,
                        fremote,
                        fowner,
                        frepository,
                        None,
                        premote,
                        powner,
                        prepository,
                        None,
                    )
                }
            }),
        )
        // Function-Profile Pairs - get usage (fcommit only)
        .route(
            "/functions/{fremote}/{fowner}/{frepository}/{fcommit}/profiles/{premote}/{powner}/{prepository}/usage",
            axum::routing::get({
                let pairs_client = pairs_client.clone();
                move |Path((fremote, fowner, frepository, fcommit, premote, powner, prepository)): Path<(objectiveai::functions::Remote, String, String, String, objectiveai::functions::Remote, String, String)>, headers: axum::http::HeaderMap| {
                    get_function_profile_pair_usage(
                        pairs_client,
                        headers,
                        fremote,
                        fowner,
                        frepository,
                        Some(fcommit),
                        premote,
                        powner,
                        prepository,
                        None,
                    )
                }
            }),
        )
        // Function-Profile Pairs - get usage (pcommit only)
        .route(
            "/functions/{fremote}/{fowner}/{frepository}/profiles/{premote}/{powner}/{prepository}/{pcommit}/usage",
            axum::routing::get({
                let pairs_client = pairs_client.clone();
                move |Path((fremote, fowner, frepository, premote, powner, prepository, pcommit)): Path<(objectiveai::functions::Remote, String, String, objectiveai::functions::Remote, String, String, String)>, headers: axum::http::HeaderMap| {
                    get_function_profile_pair_usage(
                        pairs_client,
                        headers,
                        fremote,
                        fowner,
                        frepository,
                        None,
                        premote,
                        powner,
                        prepository,
                        Some(pcommit),
                    )
                }
            }),
        )
        // Function-Profile Pairs - get usage (both commits)
        .route(
            "/functions/{fremote}/{fowner}/{frepository}/{fcommit}/profiles/{premote}/{powner}/{prepository}/{pcommit}/usage",
            axum::routing::get({
                let pairs_client = pairs_client.clone();
                move |Path((fremote, fowner, frepository, fcommit, premote, powner, prepository, pcommit)): Path<(objectiveai::functions::Remote, String, String, String, objectiveai::functions::Remote, String, String, String)>, headers: axum::http::HeaderMap| {
                    get_function_profile_pair_usage(
                        pairs_client,
                        headers,
                        fremote,
                        fowner,
                        frepository,
                        Some(fcommit),
                        premote,
                        powner,
                        prepository,
                        Some(pcommit),
                    )
                }
            }),
        )
        // Function Inventions - create
        .route(
            "/functions/inventions",
            axum::routing::post({
                let function_inventions_client = function_inventions_client.clone();
                move |headers: axum::http::HeaderMap, Json(body): Json<
                    objectiveai::functions::inventions::request::FunctionInventionCreateParams,
                >| {
                    create_function_invention(function_inventions_client, headers, body)
                }
            }),
        )
        // Function Inventions Recursive - create
        .route(
            "/functions/inventions/recursive",
            axum::routing::post({
                let function_inventions_recursive_client =
                    function_inventions_recursive_client.clone();
                move |headers: axum::http::HeaderMap, Json(body): Json<
                    objectiveai::functions::inventions::recursive::request::FunctionInventionRecursiveCreateParams,
                >| {
                    create_function_invention_recursive(
                        function_inventions_recursive_client,
                        headers,
                        body,
                    )
                }
            }),
        )
        // Function Profile Computations - create
        // inline function
        .route(
            "/functions/profiles/compute",
            axum::routing::post({
                let profile_computations_client =
                    profile_computations_client.clone();
                move |headers: axum::http::HeaderMap, Json(body): Json<
                    objectiveai::functions::profiles::computations::request::FunctionInlineRequestBody,
                >| {
                    create_profile_computation(
                        profile_computations_client,
                        headers,
                        objectiveai::functions::profiles::computations::request::Request::FunctionInline {
                            body,
                        },
                    )
                }
            }),
        )
        // Function Profile Computations - create
        // remote function (without commit)
        .route(
            "/functions/{fremote}/{fowner}/{frepository}/profiles/compute",
            axum::routing::post({
                let profile_computations_client =
                    profile_computations_client.clone();
                move |Path(path): Path<
                    objectiveai::functions::profiles::computations::request::FunctionRemoteRequestPath,
                >,
                      headers: axum::http::HeaderMap,
                      Json(body): Json<
                    objectiveai::functions::profiles::computations::request::FunctionRemoteRequestBody,
                >| {
                    create_profile_computation(
                        profile_computations_client,
                        headers,
                        objectiveai::functions::profiles::computations::request::Request::FunctionRemote {
                            path,
                            body,
                        },
                    )
                }
            }),
        )
        // Function Profile Computations - create
        // remote function (with commit)
        .route(
            "/functions/{fremote}/{fowner}/{frepository}/{fcommit}/profiles/compute",
            axum::routing::post({
                let profile_computations_client =
                    profile_computations_client.clone();
                move |Path(path): Path<
                    objectiveai::functions::profiles::computations::request::FunctionRemoteRequestPath,
                >,
                      headers: axum::http::HeaderMap,
                      Json(body): Json<
                    objectiveai::functions::profiles::computations::request::FunctionRemoteRequestBody,
                >| {
                    create_profile_computation(
                        profile_computations_client,
                        headers,
                        objectiveai::functions::profiles::computations::request::Request::FunctionRemote {
                            path,
                            body,
                        },
                    )
                }
            }),
        )
        // Auth - create API key
        .route(
            "/auth/keys",
            axum::routing::post({
                let auth_client = auth_client.clone();
                move |headers: axum::http::HeaderMap, Json(body): Json<
                    objectiveai::auth::request::CreateApiKeyRequest,
                >| {
                    create_api_key(auth_client, headers, body)
                }
            }),
        )
        // Auth - create OpenRouter BYOK API key
        .route(
            "/auth/keys/openrouter",
            axum::routing::post({
                let auth_client = auth_client.clone();
                move |headers: axum::http::HeaderMap, Json(body): Json<
                    objectiveai::auth::request::CreateOpenRouterByokApiKeyRequest,
                >| {
                    create_openrouter_byok_api_key(auth_client, headers, body)
                }
            }),
        )
        // Auth - disable API key
        .route(
            "/auth/keys",
            axum::routing::delete({
                let auth_client = auth_client.clone();
                move |headers: axum::http::HeaderMap, Json(body): Json<
                    objectiveai::auth::request::DisableApiKeyRequest,
                >| {
                    disable_api_key(auth_client, headers, body)
                }
            }),
        )
        // Auth - delete OpenRouter BYOK API key
        .route(
            "/auth/keys/openrouter",
            axum::routing::delete({
                let auth_client = auth_client.clone();
                move |headers: axum::http::HeaderMap| {
                    delete_openrouter_byok_api_key(auth_client, headers)
                }
            }),
        )
        // Auth - list API keys
        .route(
            "/auth/keys",
            axum::routing::get({
                let auth_client = auth_client.clone();
                move |headers: axum::http::HeaderMap| {
                    list_api_keys(auth_client, headers)
                }
            }),
        )
        // Auth - get OpenRouter BYOK API key
        .route(
            "/auth/keys/openrouter",
            axum::routing::get({
                let auth_client = auth_client.clone();
                move |headers: axum::http::HeaderMap| {
                    get_openrouter_byok_api_key(auth_client, headers)
                }
            }),
        )
        // Auth - get credits
        .route(
            "/auth/credits",
            axum::routing::get({
                let auth_client = auth_client.clone();
                move |headers: axum::http::HeaderMap| {
                    get_credits(auth_client, headers)
                }
            }),
        )
        // Swarm - list
        .route(
            "/swarms",
            axum::routing::get({
                let swarm_client = swarm_client.clone();
                move |headers: axum::http::HeaderMap| {
                    list_swarms(swarm_client, headers)
                }
            }),
        )
        // Swarm - get
        .route(
            "/swarms/{id}",
            axum::routing::get({
                let swarm_client = swarm_client.clone();
                move |Path(id): Path<String>, headers: axum::http::HeaderMap| {
                    get_swarm(swarm_client, headers, id)
                }
            }),
        )
        // Swarm - get usage
        .route(
            "/swarms/{id}/usage",
            axum::routing::get({
                let swarm_client = swarm_client.clone();
                move |Path(id): Path<String>, headers: axum::http::HeaderMap| {
                    get_swarm_usage(swarm_client, headers, id)
                }
            }),
        )
        // Agent - list
        .route(
            "/agents",
            axum::routing::get({
                let agent_client = agent_client.clone();
                move |headers: axum::http::HeaderMap| {
                    list_agents(agent_client, headers)
                }
            }),
        )
        // Agent - get
        .route(
            "/agents/{id}",
            axum::routing::get({
                let agent_client = agent_client.clone();
                move |Path(id): Path<String>, headers: axum::http::HeaderMap| {
                    get_agent(agent_client, headers, id)
                }
            }),
        )
        // Agent - get usage
        .route(
            "/agents/{id}/usage",
            axum::routing::get({
                let agent_client = agent_client.clone();
                move |Path(id): Path<String>, headers: axum::http::HeaderMap| {
                    get_agent_usage(agent_client, headers, id)
                }
            }),
        )
        // Error - create
        .route(
            "/error",
            axum::routing::post({
                let error_client = Arc::new(objectiveai_api::error::Client::new());
                move |Json(body): Json<
                    objectiveai::error::request::ErrorCreateParams,
                >| {
                    create_error(error_client, body)
                }
            }),
        )
        // CORS
        .layer(
            tower_http::cors::CorsLayer::new()
                .allow_origin(tower_http::cors::Any)
                .allow_methods(tower_http::cors::Any)
                .allow_headers(tower_http::cors::Any)
                .expose_headers(tower_http::cors::Any),
        );

    let listener =
        tokio::net::TcpListener::bind(format!("{}:{}", address, port))
            .await
            .unwrap();

    eprintln!("listening on {}:{}", address, port);
    axum::serve(listener, app).await.unwrap();
}

// Create Context

fn context(headers: &axum::http::HeaderMap) -> ctx::Context<ctx::DefaultContextExt> {
    ctx::Context::new(
        Arc::new(ctx::DefaultContextExt),
        rust_decimal::Decimal::ONE,
        headers,
    )
}

// Agent Completions

async fn create_agent_completion(
    client: Arc<
        agent::completions::Client<
            ctx::DefaultContextExt,
            impl agent::completions::UpstreamClient<
                objectiveai::agent::openrouter::Agent,
            > + Send
            + Sync
            + 'static,
            impl agent::completions::UpstreamClient<
                objectiveai::agent::claude_agent_sdk::Agent,
            > + Send
            + Sync
            + 'static,
            impl agent::completions::UpstreamClient<
                objectiveai::agent::mock::Agent,
            > + Send
            + Sync
            + 'static,
            impl agent::fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl agent::completions::usage_handler::UsageHandler<
                ctx::DefaultContextExt,
            > + Send
            + Sync
            + 'static,
        >,
    >,
    headers: axum::http::HeaderMap,
    body: objectiveai::agent::completions::request::AgentCompletionCreateParams,
) -> axum::response::Response {
    let ctx = context(&headers);
    if body.stream.unwrap_or(false) {
        match client
            .create_streaming_handle_usage(
                ctx,
                Arc::new(body),
                None,
                None,
                None,
                None,
            )
            .await
        {
            Ok(stream) => Sse::new(
                stream
                    .filter_map(|item| {
                        match item {
                            agent::completions::StreamItem::Chunk(chunk) => {
                                Some(Ok::<Event, Infallible>(
                                    Event::default()
                                        .data(serde_json::to_string(&chunk).unwrap()),
                                ))
                            }
                            agent::completions::StreamItem::State(_) => None,
                        }
                    })
                    .chain(StreamOnce::new(
                        Ok(Event::default().data("[DONE]")),
                    )),
            )
            .into_response(),
            Err(e) => ResponseError::from(&e).into_response(),
        }
    } else {
        match client
            .create_unary_handle_usage(
                ctx,
                Arc::new(body),
                None,
                None,
                None,
                None,
            )
            .await
        {
            Ok(r) => Json(r).into_response(),
            Err(e) => ResponseError::from(&e).into_response(),
        }
    }
}

// Vector Completions

async fn create_vector_completion(
    client: Arc<
        vector::completions::Client<
            ctx::DefaultContextExt,
            impl agent::completions::UpstreamClient<
                objectiveai::agent::openrouter::Agent,
            > + Send
            + Sync
            + 'static,
            impl agent::completions::UpstreamClient<
                objectiveai::agent::claude_agent_sdk::Agent,
            > + Send
            + Sync
            + 'static,
            impl agent::completions::UpstreamClient<
                objectiveai::agent::mock::Agent,
            > + Send
            + Sync
            + 'static,
            impl agent::fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl agent::completions::usage_handler::UsageHandler<
                ctx::DefaultContextExt,
            > + Send
            + Sync
            + 'static,
            impl swarm::fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl vector::completions::completion_votes_fetcher::Fetcher<
                ctx::DefaultContextExt,
            > + Send
            + Sync
            + 'static,
            impl vector::completions::cache_vote_fetcher::Fetcher<
                ctx::DefaultContextExt,
            > + Send
            + Sync
            + 'static,
            impl vector::completions::usage_handler::UsageHandler<
                ctx::DefaultContextExt,
            > + Send
            + Sync
            + 'static,
        >,
    >,
    headers: axum::http::HeaderMap,
    body: objectiveai::vector::completions::request::VectorCompletionCreateParams,
) -> axum::response::Response {
    let ctx = context(&headers);
    if body.stream.unwrap_or(false) {
        match client
            .create_streaming_handle_usage(ctx, Arc::new(body))
            .await
        {
            Ok(stream) => Sse::new(
                stream
                    .map(|chunk| {
                        Ok::<Event, Infallible>(
                            Event::default()
                                .data(serde_json::to_string(&chunk).unwrap()),
                        )
                    })
                    .chain(StreamOnce::new(
                        Ok(Event::default().data("[DONE]")),
                    )),
            )
            .into_response(),
            Err(e) => ResponseError::from(&e).into_response(),
        }
    } else {
        match client.create_unary_handle_usage(ctx, Arc::new(body)).await {
            Ok(r) => Json(r).into_response(),
            Err(e) => ResponseError::from(&e).into_response(),
        }
    }
}

// Functions

async fn list_functions(
    client: Arc<
        functions::Client<
            ctx::DefaultContextExt,
            impl functions::function_fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl functions::function_fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl functions::function_fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl functions::retrieval_client::Client<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
        >,
    >,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    let ctx = context(&headers);
    match client.list_functions(ctx).await {
        Ok(r) => Json(r).into_response(),
        Err(e) => ResponseError::from(&e).into_response(),
    }
}

async fn get_function_usage(
    client: Arc<
        functions::Client<
            ctx::DefaultContextExt,
            impl functions::function_fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl functions::function_fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl functions::function_fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl functions::retrieval_client::Client<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
        >,
    >,
    headers: axum::http::HeaderMap,
    remote: objectiveai::functions::Remote,
    owner: String,
    repository: String,
    commit: Option<String>,
) -> axum::response::Response {
    let ctx = context(&headers);
    match client
        .get_function_usage(ctx, remote, &owner, &repository, commit.as_deref())
        .await
    {
        Ok(r) => Json(r).into_response(),
        Err(e) => ResponseError::from(&e).into_response(),
    }
}

async fn execute_function(
    client: Arc<
        functions::executions::Client<
            ctx::DefaultContextExt,
            impl agent::completions::UpstreamClient<
                objectiveai::agent::openrouter::Agent,
            > + Send
            + Sync
            + 'static,
            impl agent::completions::UpstreamClient<
                objectiveai::agent::claude_agent_sdk::Agent,
            > + Send
            + Sync
            + 'static,
            impl agent::completions::UpstreamClient<
                objectiveai::agent::mock::Agent,
            > + Send
            + Sync
            + 'static,
            impl agent::fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl agent::completions::usage_handler::UsageHandler<
                ctx::DefaultContextExt,
            > + Send
            + Sync
            + 'static,
            impl swarm::fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl vector::completions::completion_votes_fetcher::Fetcher<
                ctx::DefaultContextExt,
            > + Send
            + Sync
            + 'static,
            impl vector::completions::cache_vote_fetcher::Fetcher<
                ctx::DefaultContextExt,
            > + Send
            + Sync
            + 'static,
            impl vector::completions::usage_handler::UsageHandler<
                ctx::DefaultContextExt,
            > + Send
            + Sync
            + 'static,
            impl functions::function_fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl functions::function_fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl functions::function_fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl functions::profile_fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl functions::profile_fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl functions::profile_fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl functions::executions::usage_handler::UsageHandler<
                ctx::DefaultContextExt,
            > + Send
            + Sync
            + 'static,
        >,
    >,
    headers: axum::http::HeaderMap,
    request: objectiveai::functions::executions::request::Request,
) -> axum::response::Response {
    let ctx = context(&headers);
    if request.base().stream.unwrap_or(false) {
        match client
            .create_streaming_handle_usage(ctx, Arc::new(request))
            .await
        {
            Ok(stream) => Sse::new(
                stream
                    .map(|chunk| {
                        Ok::<Event, Infallible>(
                            Event::default()
                                .data(serde_json::to_string(&chunk).unwrap()),
                        )
                    })
                    .chain(StreamOnce::new(
                        Ok(Event::default().data("[DONE]")),
                    )),
            )
            .into_response(),
            Err(e) => ResponseError::from(&e).into_response(),
        }
    } else {
        match client
            .create_unary_handle_usage(ctx, Arc::new(request))
            .await
        {
            Ok(r) => Json(r).into_response(),
            Err(e) => ResponseError::from(&e).into_response(),
        }
    }
}

// Profiles

async fn list_profiles(
    client: Arc<
        functions::profiles::Client<
            ctx::DefaultContextExt,
            impl functions::profile_fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl functions::profile_fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl functions::profile_fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl functions::profiles::retrieval_client::Client<
                ctx::DefaultContextExt,
            > + Send
            + Sync
            + 'static,
        >,
    >,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    let ctx = context(&headers);
    match client.list_profiles(ctx).await {
        Ok(r) => Json(r).into_response(),
        Err(e) => ResponseError::from(&e).into_response(),
    }
}

async fn get_profile_usage(
    client: Arc<
        functions::profiles::Client<
            ctx::DefaultContextExt,
            impl functions::profile_fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl functions::profile_fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl functions::profile_fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl functions::profiles::retrieval_client::Client<
                ctx::DefaultContextExt,
            > + Send
            + Sync
            + 'static,
        >,
    >,
    headers: axum::http::HeaderMap,
    remote: objectiveai::functions::Remote,
    owner: String,
    repository: String,
    commit: Option<String>,
) -> axum::response::Response {
    let ctx = context(&headers);
    match client
        .get_profile_usage(ctx, remote, &owner, &repository, commit.as_deref())
        .await
    {
        Ok(r) => Json(r).into_response(),
        Err(e) => ResponseError::from(&e).into_response(),
    }
}

// Function-Profile Pairs

async fn list_function_profile_pairs(
    client: Arc<
        impl functions::pair_retrieval_client::Client<ctx::DefaultContextExt>
        + Send
        + Sync
        + 'static,
    >,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    let ctx = context(&headers);
    match client.list_function_profile_pairs(ctx).await {
        Ok(r) => Json(r).into_response(),
        Err(e) => e.into_response(),
    }
}


async fn get_function_profile_pair_usage(
    client: Arc<
        impl functions::pair_retrieval_client::Client<ctx::DefaultContextExt>
        + Send
        + Sync
        + 'static,
    >,
    headers: axum::http::HeaderMap,
    fremote: objectiveai::functions::Remote,
    fowner: String,
    frepository: String,
    fcommit: Option<String>,
    premote: objectiveai::functions::Remote,
    powner: String,
    prepository: String,
    pcommit: Option<String>,
) -> axum::response::Response {
    let ctx = context(&headers);
    match client
        .get_function_profile_pair_usage(
            ctx,
            fremote,
            &fowner,
            &frepository,
            fcommit.as_deref(),
            premote,
            &powner,
            &prepository,
            pcommit.as_deref(),
        )
        .await
    {
        Ok(r) => Json(r).into_response(),
        Err(e) => e.into_response(),
    }
}

// Vector Completions Cache

async fn get_vector_completion_votes(
    client: Arc<
        vector::completions::cache::Client<
            ctx::DefaultContextExt,
            impl vector::completions::completion_votes_fetcher::Fetcher<
                ctx::DefaultContextExt,
            > + Send
            + Sync
            + 'static,
            impl vector::completions::cache_vote_fetcher::Fetcher<
                ctx::DefaultContextExt,
            > + Send
            + Sync
            + 'static,
        >,
    >,
    headers: axum::http::HeaderMap,
    id: String,
) -> axum::response::Response {
    let ctx = context(&headers);
    match client.fetch_completion_votes(ctx, &id).await {
        Ok(r) => Json(r).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn get_vector_cache_vote(
    client: Arc<
        vector::completions::cache::Client<
            ctx::DefaultContextExt,
            impl vector::completions::completion_votes_fetcher::Fetcher<
                ctx::DefaultContextExt,
            > + Send
            + Sync
            + 'static,
            impl vector::completions::cache_vote_fetcher::Fetcher<
                ctx::DefaultContextExt,
            > + Send
            + Sync
            + 'static,
        >,
    >,
    headers: axum::http::HeaderMap,
    body: objectiveai::vector::completions::cache::request::CacheVoteRequestOwned,
) -> axum::response::Response {
    let ctx = context(&headers);
    match client
        .fetch_cache_vote(
            ctx,
            &body.agent,
            body.agents.as_deref(),
            &body.messages,
            &body.responses,
        )
        .await
    {
        Ok(r) => Json(r).into_response(),
        Err(e) => e.into_response(),
    }
}

// Functions - get

async fn get_function(
    client: Arc<
        functions::Client<
            ctx::DefaultContextExt,
            impl functions::function_fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl functions::function_fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl functions::function_fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl functions::retrieval_client::Client<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
        >,
    >,
    headers: axum::http::HeaderMap,
    remote: objectiveai::functions::Remote,
    owner: String,
    repository: String,
    commit: Option<String>,
) -> axum::response::Response {
    let ctx = context(&headers);
    match client
        .get_function(ctx, remote, &owner, &repository, commit.as_deref())
        .await
    {
        Ok(r) => Json(r).into_response(),
        Err(e) => e.into_response(),
    }
}

// Profiles - get

async fn get_profile(
    client: Arc<
        functions::profiles::Client<
            ctx::DefaultContextExt,
            impl functions::profile_fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl functions::profile_fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl functions::profile_fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl functions::profiles::retrieval_client::Client<
                ctx::DefaultContextExt,
            > + Send
            + Sync
            + 'static,
        >,
    >,
    headers: axum::http::HeaderMap,
    remote: objectiveai::functions::Remote,
    owner: String,
    repository: String,
    commit: Option<String>,
) -> axum::response::Response {
    let ctx = context(&headers);
    match client
        .get_profile(ctx, remote, &owner, &repository, commit.as_deref())
        .await
    {
        Ok(r) => Json(r).into_response(),
        Err(e) => e.into_response(),
    }
}

// Profile Computations

async fn create_profile_computation(
    // client: Arc<
    //     impl functions::profiles::computations::Client<ctx::DefaultContextExt>
    //     + Send
    //     + Sync
    //     + 'static,
    // >,
    // https://github.com/rust-lang/rust/issues/100013
    // using a concrete type for client instead
    client: Arc<functions::profiles::computations::ObjectiveAiClient>,
    headers: axum::http::HeaderMap,
    request: objectiveai::functions::profiles::computations::request::Request,
) -> axum::response::Response {
    let ctx = context(&headers);
    if request.base().stream.unwrap_or(false) {
        match client.create_streaming(ctx, Arc::new(request)).await {
            Ok(stream) => Sse::new(
                stream
                    .map(|result| {
                        Ok::<Event, Infallible>(
                            Event::default().data(
                                match result {
                                    Ok(chunk) => serde_json::to_string(&chunk),
                                    Err(e) => serde_json::to_string(&e),
                                }
                                .unwrap(),
                            ),
                        )
                    })
                    .chain(StreamOnce::new(
                        Ok(Event::default().data("[DONE]")),
                    )),
            )
            .into_response(),
            Err(e) => e.into_response(),
        }
    } else {
        match client.create_unary(ctx, Arc::new(request)).await {
            Ok(r) => Json(r).into_response(),
            Err(e) => e.into_response(),
        }
    }
}

// Auth

async fn create_api_key(
    client: Arc<
        impl auth::Client<ctx::DefaultContextExt> + Send + Sync + 'static,
    >,
    headers: axum::http::HeaderMap,
    body: objectiveai::auth::request::CreateApiKeyRequest,
) -> axum::response::Response {
    let ctx = context(&headers);
    match client.create_api_key(ctx, body).await {
        Ok(r) => Json(r).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn create_openrouter_byok_api_key(
    client: Arc<
        impl auth::Client<ctx::DefaultContextExt> + Send + Sync + 'static,
    >,
    headers: axum::http::HeaderMap,
    body: objectiveai::auth::request::CreateOpenRouterByokApiKeyRequest,
) -> axum::response::Response {
    let ctx = context(&headers);
    match client.create_openrouter_byok_api_key(ctx, body).await {
        Ok(r) => Json(r).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn disable_api_key(
    client: Arc<
        impl auth::Client<ctx::DefaultContextExt> + Send + Sync + 'static,
    >,
    headers: axum::http::HeaderMap,
    body: objectiveai::auth::request::DisableApiKeyRequest,
) -> axum::response::Response {
    let ctx = context(&headers);
    match client.disable_api_key(ctx, body).await {
        Ok(r) => Json(r).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn delete_openrouter_byok_api_key(
    client: Arc<
        impl auth::Client<ctx::DefaultContextExt> + Send + Sync + 'static,
    >,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    let ctx = context(&headers);
    match client.delete_openrouter_byok_api_key(ctx).await {
        Ok(()) => axum::http::StatusCode::OK.into_response(),
        Err(e) => e.into_response(),
    }
}

async fn list_api_keys(
    client: Arc<
        impl auth::Client<ctx::DefaultContextExt> + Send + Sync + 'static,
    >,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    let ctx = context(&headers);
    match client.list_api_keys(ctx).await {
        Ok(r) => Json(r).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn get_openrouter_byok_api_key(
    client: Arc<
        impl auth::Client<ctx::DefaultContextExt> + Send + Sync + 'static,
    >,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    let ctx = context(&headers);
    match client.get_openrouter_byok_api_key(ctx).await {
        Ok(r) => Json(r).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn get_credits(
    client: Arc<
        impl auth::Client<ctx::DefaultContextExt> + Send + Sync + 'static,
    >,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    let ctx = context(&headers);
    match client.get_credits(ctx).await {
        Ok(r) => Json(r).into_response(),
        Err(e) => e.into_response(),
    }
}

// Swarm

async fn list_swarms(
    client: Arc<
        swarm::Client<
            ctx::DefaultContextExt,
            impl swarm::fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl swarm::retrieval_client::Client<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
        >,
    >,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    let ctx = context(&headers);
    match client.list(ctx).await {
        Ok(r) => Json(r).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn get_swarm(
    client: Arc<
        swarm::Client<
            ctx::DefaultContextExt,
            impl swarm::fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl swarm::retrieval_client::Client<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
        >,
    >,
    headers: axum::http::HeaderMap,
    id: String,
) -> axum::response::Response {
    let ctx = context(&headers);
    match client.get(ctx, &id).await {
        Ok(r) => Json(r).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn get_swarm_usage(
    client: Arc<
        swarm::Client<
            ctx::DefaultContextExt,
            impl swarm::fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl swarm::retrieval_client::Client<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
        >,
    >,
    headers: axum::http::HeaderMap,
    id: String,
) -> axum::response::Response {
    let ctx = context(&headers);
    match client.get_usage(ctx, &id).await {
        Ok(r) => Json(r).into_response(),
        Err(e) => e.into_response(),
    }
}

// Agent

async fn list_agents(
    client: Arc<
        agent::Client<
            ctx::DefaultContextExt,
            impl agent::fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl agent::retrieval_client::Client<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
        >,
    >,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    let ctx = context(&headers);
    match client.list(ctx).await {
        Ok(r) => Json(r).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn get_agent(
    client: Arc<
        agent::Client<
            ctx::DefaultContextExt,
            impl agent::fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl agent::retrieval_client::Client<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
        >,
    >,
    headers: axum::http::HeaderMap,
    id: String,
) -> axum::response::Response {
    let ctx = context(&headers);
    match client.get(ctx, &id).await {
        Ok(r) => Json(r).into_response(),
        Err(e) => e.into_response(),
    }
}

async fn get_agent_usage(
    client: Arc<
        agent::Client<
            ctx::DefaultContextExt,
            impl agent::fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl agent::retrieval_client::Client<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
        >,
    >,
    headers: axum::http::HeaderMap,
    id: String,
) -> axum::response::Response {
    let ctx = context(&headers);
    match client.get_usage(ctx, &id).await {
        Ok(r) => Json(r).into_response(),
        Err(e) => e.into_response(),
    }
}
// Function Inventions

async fn create_function_invention(
    client: Arc<
        functions::inventions::Client<
            ctx::DefaultContextExt,
            impl agent::completions::UpstreamClient<
                objectiveai::agent::openrouter::Agent,
            > + Send
            + Sync
            + 'static,
            impl agent::completions::UpstreamClient<
                objectiveai::agent::claude_agent_sdk::Agent,
            > + Send
            + Sync
            + 'static,
            impl agent::completions::UpstreamClient<
                objectiveai::agent::mock::Agent,
            > + Send
            + Sync
            + 'static,
            impl agent::fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl agent::completions::usage_handler::UsageHandler<
                ctx::DefaultContextExt,
            > + Send
            + Sync
            + 'static,
            impl functions::inventions::usage_handler::UsageHandler<
                ctx::DefaultContextExt,
            > + Send
            + Sync
            + 'static,
            impl functions::function_fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl functions::function_fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl functions::function_fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
        >,
    >,
    headers: axum::http::HeaderMap,
    body: objectiveai::functions::inventions::request::FunctionInventionCreateParams,
) -> axum::response::Response {
    let ctx = context(&headers);
    if body.stream.unwrap_or(false) {
        match client
            .create_streaming_handle_usage(ctx, Arc::new(body))
            .await
        {
            Ok(stream) => Sse::new(
                stream
                    .map(|chunk| {
                        Ok::<Event, Infallible>(
                            Event::default()
                                .data(serde_json::to_string(&chunk).unwrap()),
                        )
                    })
                    .chain(StreamOnce::new(
                        Ok(Event::default().data("[DONE]")),
                    )),
            )
            .into_response(),
            Err(e) => ResponseError::from(&e).into_response(),
        }
    } else {
        match client
            .create_unary_handle_usage(ctx, Arc::new(body))
            .await
        {
            Ok(r) => Json(r).into_response(),
            Err(e) => ResponseError::from(&e).into_response(),
        }
    }
}

// Function Inventions Recursive

async fn create_function_invention_recursive(
    client: Arc<
        functions::inventions::recursive::Client<
            ctx::DefaultContextExt,
            impl agent::completions::UpstreamClient<
                objectiveai::agent::openrouter::Agent,
            > + Send
            + Sync
            + 'static,
            impl agent::completions::UpstreamClient<
                objectiveai::agent::claude_agent_sdk::Agent,
            > + Send
            + Sync
            + 'static,
            impl agent::completions::UpstreamClient<
                objectiveai::agent::mock::Agent,
            > + Send
            + Sync
            + 'static,
            impl agent::fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl agent::completions::usage_handler::UsageHandler<
                ctx::DefaultContextExt,
            > + Send
            + Sync
            + 'static,
            impl functions::inventions::usage_handler::UsageHandler<
                ctx::DefaultContextExt,
            > + Send
            + Sync
            + 'static,
            impl functions::function_fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl functions::function_fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl functions::function_fetcher::Fetcher<ctx::DefaultContextExt>
            + Send
            + Sync
            + 'static,
            impl functions::inventions::recursive::usage_handler::UsageHandler<
                ctx::DefaultContextExt,
            > + Send
            + Sync
            + 'static,
        >,
    >,
    headers: axum::http::HeaderMap,
    body: objectiveai::functions::inventions::recursive::request::FunctionInventionRecursiveCreateParams,
) -> axum::response::Response {
    let ctx = context(&headers);
    if body.stream.unwrap_or(false) {
        match client
            .create_streaming_handle_usage(ctx, Arc::new(body))
            .await
        {
            Ok(stream) => Sse::new(
                stream
                    .map(|chunk| {
                        Ok::<Event, Infallible>(
                            Event::default()
                                .data(serde_json::to_string(&chunk).unwrap()),
                        )
                    })
                    .chain(StreamOnce::new(
                        Ok(Event::default().data("[DONE]")),
                    )),
            )
            .into_response(),
            Err(e) => ResponseError::from(&e).into_response(),
        }
    } else {
        match client
            .create_unary_handle_usage(ctx, Arc::new(body))
            .await
        {
            Ok(r) => Json(r).into_response(),
            Err(e) => ResponseError::from(&e).into_response(),
        }
    }
}

// Error

async fn create_error(
    client: Arc<objectiveai_api::error::Client>,
    body: objectiveai::error::request::ErrorCreateParams,
) -> axum::response::Response {
    if body.stream.unwrap_or(false) {
        match client.create_streaming(&body) {
            Ok(stream) => Sse::new(
                stream
                    .map(|result| {
                        Ok::<Event, Infallible>(
                            Event::default().data(
                                match result {
                                    Ok(chunk) => serde_json::to_string(&chunk),
                                    Err(e) => serde_json::to_string(&e),
                                }
                                .unwrap(),
                            ),
                        )
                    })
                    .chain(StreamOnce::new(
                        Ok(Event::default().data("[DONE]")),
                    )),
            )
            .into_response(),
            Err(e) => e.into_response(),
        }
    } else {
        match client.create_unary(&body) {
            Ok(r) => Json(r).into_response(),
            Err(e) => e.into_response(),
        }
    }
}
