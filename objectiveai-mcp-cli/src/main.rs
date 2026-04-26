use std::sync::Arc;
use envconfig::Envconfig;
use rmcp::{
    ServerHandler,
    handler::server::router::tool::ToolRouter,
    handler::server::wrapper::Parameters,
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
};
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService,
    session::local::LocalSessionManager,
};
use tokio_util::sync::CancellationToken;

#[derive(Envconfig)]
struct EnvConfigBuilder {
    #[envconfig(from = "ADDRESS")]
    address: Option<String>,
    #[envconfig(from = "PORT")]
    port: Option<u16>,
}

impl EnvConfigBuilder {
    fn build(self) -> ConfigBuilder {
        ConfigBuilder {
            address: self.address,
            port: self.port,
        }
    }
}

#[derive(Default)]
struct ConfigBuilder {
    address: Option<String>,
    port: Option<u16>,
}

impl Envconfig for ConfigBuilder {
    #[allow(deprecated)]
    fn init() -> Result<Self, envconfig::Error> {
        EnvConfigBuilder::init().map(|e| e.build())
    }

    fn init_from_env() -> Result<Self, envconfig::Error> {
        EnvConfigBuilder::init_from_env().map(|e| e.build())
    }

    fn init_from_hashmap(
        hashmap: &std::collections::HashMap<String, String>,
    ) -> Result<Self, envconfig::Error> {
        EnvConfigBuilder::init_from_hashmap(hashmap).map(|e| e.build())
    }
}

impl ConfigBuilder {
    fn build(self) -> Config {
        Config {
            address: self.address.unwrap_or_else(|| "0.0.0.0".into()),
            port: self.port.unwrap_or(3000),
        }
    }
}

struct Config {
    address: String,
    port: u16,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
struct ObjectiveAiRequest {
    #[schemars(description = "The command arguments to pass to the ObjectiveAI CLI (e.g. [\"agents\", \"list\"] or [\"functions\", \"executions\", \"create\", \"--help\"])")]
    command: Vec<String>,
}

#[derive(Debug, Clone)]
struct ObjectiveAiMcpCli {
    tool_router: ToolRouter<Self>,
    cli_config: Arc<objectiveai_cli::Config>,
}

#[tool_router]
impl ObjectiveAiMcpCli {
    fn new(cli_config: Arc<objectiveai_cli::Config>) -> Self {
        Self {
            tool_router: Self::tool_router(),
            cli_config,
        }
    }

    #[tool(
        name = "ObjectiveAI",
        description = "Run an ObjectiveAI CLI command. Supports all subcommands: agents, swarms, functions, api, schemas, viewer."
    )]
    async fn objectiveai(
        &self,
        Parameters(req): Parameters<ObjectiveAiRequest>,
    ) -> String {
        let args: Vec<String> = std::iter::once("objectiveai".to_string())
            .chain(req.command)
            .collect();

        match objectiveai_cli::run(args, &self.cli_config).await {
            Ok(output) => output,
            Err(e) => format!("error: {e}"),
        }
    }
}

#[tool_handler]
impl ServerHandler for ObjectiveAiMcpCli {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("ObjectiveAI CLI MCP server".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::DEBUG.into()),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    let _ = dotenv::dotenv();
    let config = ConfigBuilder::init_from_env()
        .unwrap_or_default()
        .build();
    let cli_config = Arc::new(
        objectiveai_cli::ConfigBuilder::init_from_env()
            .unwrap_or_default()
            .build(),
    );

    tracing::info!(
        "Starting ObjectiveAI CLI MCP server on {}:{}",
        config.address,
        config.port,
    );

    let server = ObjectiveAiMcpCli::new(cli_config);
    let ct = CancellationToken::new();

    let service: StreamableHttpService<ObjectiveAiMcpCli, LocalSessionManager> =
        StreamableHttpService::new(
            move || Ok(server.clone()),
            Default::default(),
            StreamableHttpServerConfig {
                stateful_mode: true,
                sse_keep_alive: None,
                cancellation_token: ct.child_token(),
                ..Default::default()
            },
        );

    let router = axum::Router::new().nest_service("/", service);
    let listener = tokio::net::TcpListener::bind(format!(
        "{}:{}",
        config.address, config.port,
    ))
    .await?;
    tracing::info!("Listening on {}", listener.local_addr()?);

    axum::serve(listener, router)
        .with_graceful_shutdown(async move { ct.cancelled_owned().await })
        .await?;

    Ok(())
}
