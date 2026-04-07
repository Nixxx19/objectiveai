#[cfg(feature = "filesystem")]
mod filesystem;

#[cfg(feature = "cli")]
mod cli;

use rmcp::{
    ServerHandler,
    handler::server::router::tool::ToolRouter,
    handler::server::tool::ToolCallContext,
    model::{CallToolResult, ServerCapabilities, ServerInfo},
    tool_handler,
    transport::stdio,
    ErrorData, ServiceExt,
};
use futures::FutureExt;

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

    tracing::info!("Starting ObjectiveAI MCP server");

    let server = ObjectiveAiMcp::new();
    server.init().await;

    let service = server
        .serve(stdio())
        .await
        .inspect_err(|e| {
            tracing::error!("serving error: {:?}", e);
        })?;

    service.waiting().await?;
    Ok(())
}
