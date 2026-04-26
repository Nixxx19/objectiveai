#[cfg(feature = "filesystem")]
mod filesystem;

#[cfg(feature = "cli")]
mod cli;

mod mcp_client;

use rmcp::{
    ServerHandler,
    handler::server::router::tool::ToolRouter,
    handler::server::tool::ToolCallContext,
    model::{ServerCapabilities, ServerInfo},
    tool_handler,
    ServiceExt,
};
use rmcp::transport::streamable_http_server::{
    StreamableHttpServerConfig, StreamableHttpService,
    session::local::LocalSessionManager,
};
use envconfig::Envconfig;
use futures::FutureExt;
use tokio_util::sync::CancellationToken;

#[derive(Envconfig)]
struct EnvConfigBuilder {
    #[envconfig(from = "ADDRESS")]
    address: Option<String>,
    #[envconfig(from = "PORT")]
    port: Option<u16>,
}

impl EnvConfigBuilder {
    pub fn build(self) -> ConfigBuilder {
        ConfigBuilder {
            address: self.address,
            port: self.port,
        }
    }
}

#[derive(Default)]
pub struct ConfigBuilder {
    pub address: Option<String>,
    pub port: Option<u16>,
}

impl Envconfig for ConfigBuilder {
    #[allow(deprecated)]
    fn init() -> Result<Self, envconfig::Error> {
        EnvConfigBuilder::init().map(|e| e.build())
    }

    fn init_from_env() -> Result<Self, envconfig::Error> {
        EnvConfigBuilder::init_from_env().map(|e| e.build())
    }

    fn init_from_hashmap(hashmap: &std::collections::HashMap<String, String>) -> Result<Self, envconfig::Error> {
        EnvConfigBuilder::init_from_hashmap(hashmap).map(|e| e.build())
    }
}

impl ConfigBuilder {
    pub fn build(self) -> Config {
        Config {
            address: self.address.unwrap_or_else(|| "0.0.0.0".to_string()),
            port: self.port.unwrap_or(3000),
        }
    }
}

pub struct Config {
    pub address: String,
    pub port: u16,
}

#[derive(Debug, Clone)]
struct ObjectiveAiMcp {
    tool_router: ToolRouter<Self>,
    #[cfg(feature = "filesystem")]
    filesystem: filesystem::FilesystemTools,
    #[cfg(feature = "cli")]
    cli: cli::CliTools,
}

impl ObjectiveAiMcp {
    fn new() -> Self {
        let mut tool_router = ToolRouter::<Self>::new();

        #[cfg(feature = "filesystem")]
        let filesystem = {
            let fs = filesystem::FilesystemTools::new();
            for tool_def in fs.tool_router.list_all() {
                let fs_router = fs.tool_router.clone();
                let fs_ref = fs.clone();
                tool_router.add_route(
                    rmcp::handler::server::router::tool::ToolRoute::new_dyn(
                        tool_def,
                        move |ctx: ToolCallContext<'_, ObjectiveAiMcp>| {
                            let fs_router = fs_router.clone();
                            let fs_ref = fs_ref.clone();
                            let params = rmcp::model::CallToolRequestParams {
                                meta: None,
                                name: ctx.name.clone(),
                                arguments: ctx.arguments.clone(),
                                task: ctx.task.clone(),
                            };
                            let request_context = ctx.request_context.clone();
                            async move {
                                let sub_ctx = ToolCallContext::new(
                                    &fs_ref,
                                    params,
                                    request_context,
                                );
                                fs_router.call(sub_ctx).await
                            }
                            .boxed()
                        },
                    ),
                );
            }
            fs
        };

        #[cfg(feature = "cli")]
        let cli_tools = {
            let ct = cli::CliTools::new();
            for tool_def in ct.tool_router.list_all() {
                let ct_router = ct.tool_router.clone();
                let ct_ref = ct.clone();
                tool_router.add_route(
                    rmcp::handler::server::router::tool::ToolRoute::new_dyn(
                        tool_def,
                        move |ctx: ToolCallContext<'_, ObjectiveAiMcp>| {
                            let ct_router = ct_router.clone();
                            let ct_ref = ct_ref.clone();
                            let params = rmcp::model::CallToolRequestParams {
                                meta: None,
                                name: ctx.name.clone(),
                                arguments: ctx.arguments.clone(),
                                task: ctx.task.clone(),
                            };
                            let request_context = ctx.request_context.clone();
                            async move {
                                let sub_ctx = ToolCallContext::new(
                                    &ct_ref,
                                    params,
                                    request_context,
                                );
                                ct_router.call(sub_ctx).await
                            }
                            .boxed()
                        },
                    ),
                );
            }
            ct
        };

        Self {
            tool_router,
            #[cfg(feature = "filesystem")]
            filesystem,
            #[cfg(feature = "cli")]
            cli: cli_tools,
        }
    }

    async fn init(&self) {
        #[cfg(feature = "filesystem")]
        self.filesystem.init().await;
    }
}

#[tool_handler]
impl ServerHandler for ObjectiveAiMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("ObjectiveAI MCP server".into()),
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

    tracing::info!("Starting ObjectiveAI MCP server on {}:{}", config.address, config.port);

    let server = ObjectiveAiMcp::new();
    server.init().await;

    let ct = CancellationToken::new();

    let service: StreamableHttpService<ObjectiveAiMcp, LocalSessionManager> =
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
    let listener = tokio::net::TcpListener::bind(
        format!("{}:{}", config.address, config.port),
    ).await?;
    tracing::info!("Listening on {}", listener.local_addr()?);

    axum::serve(listener, router)
        .with_graceful_shutdown(async move { ct.cancelled_owned().await })
        .await?;

    Ok(())
}
