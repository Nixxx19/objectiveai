use std::borrow::Cow;
use std::sync::Arc;

use futures::FutureExt;
use http::request::Parts;
use objectiveai_sdk::filesystem::plugins::ManifestWithNameAndSource as PluginManifest;
use objectiveai_sdk::filesystem::tools::ManifestWithNameAndSource as ToolManifest;
use rmcp::{
    ServerHandler,
    handler::server::router::tool::{ToolRoute, ToolRouter},
    handler::server::tool::{Extension, parse_json_object, schema_for_type},
    handler::server::wrapper::Parameters,
    model::{
        CallToolResult, Content, Implementation, ProtocolVersion, ServerCapabilities, ServerInfo,
        Tool,
    },
    schemars, tool, tool_handler, tool_router,
};

use crate::format::{OutputMode, format_outputs};

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ObjectiveAiRequest {
    #[schemars(description = "The command arguments to pass to the ObjectiveAI CLI (e.g. [\"agents\", \"list\"] or [\"functions\", \"executions\", \"create\", \"--help\"])")]
    pub command: Vec<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PluginRequest {
    #[schemars(description = "Args forwarded to the plugin's argv (prefixed automatically with `plugins <name>` when invoking the CLI).")]
    pub args: Vec<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ToolRequest {
    #[schemars(description = "Args forwarded to the tool's argv (prefixed automatically with `tools <name>` when invoking the CLI).")]
    pub args: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ObjectiveAiMcpCli {
    pub tool_router: ToolRouter<Self>,
    pub cli_config: Arc<objectiveai_cli::Config>,
    /// When `true`, [`run_cli_and_collect`] strips `agent_id` from
    /// any rendered `Output::Error` in the response body. Set via
    /// the `TEST_MODE` env var at startup. Off in production. See
    /// [`crate::format`] for which rendered shapes carry `agent_id`
    /// (only errors — notification renderings drop it by construction).
    pub test_mode: bool,
}

#[tool_router]
impl ObjectiveAiMcpCli {
    /// Build a handler with one dynamic tool per discovered CLI plugin
    /// and CLI tool, in addition to the static `ObjectiveAI` catch-all.
    /// Plugins and tools are listed once at server startup (see
    /// `run::setup`); this constructor is not re-invoked when either
    /// is added later, so hot reload is intentionally out of scope.
    ///
    /// Name collisions: if a CLI plugin and a CLI tool happen to share
    /// a name (or with `ObjectiveAI`), the plugin registers first and
    /// the tool's `add_route` overwrites it (last-writer-wins) — but
    /// `ObjectiveAI` itself is always skipped on both sides so the
    /// built-in catch-all is never shadowed.
    pub fn with_plugins_and_tools(
        cli_config: Arc<objectiveai_cli::Config>,
        plugins: Vec<PluginManifest>,
        tools: Vec<ToolManifest>,
        test_mode: bool,
    ) -> Self {
        let mut tool_router = Self::tool_router();
        for plugin in plugins {
            if plugin.name == "ObjectiveAI" {
                continue;
            }
            let plugin_name = plugin.name.clone();
            let cli_config_for_route = cli_config.clone();
            let tool = Tool::new(
                Cow::Owned(plugin.name.clone()),
                Cow::Owned(plugin.manifest.description.clone()),
                schema_for_type::<PluginRequest>(),
            );
            tool_router.add_route(ToolRoute::new_dyn(tool, move |ctx| {
                let cli_config = cli_config_for_route.clone();
                let plugin_name = plugin_name.clone();
                async move {
                    let arguments = ctx.arguments.unwrap_or_default();
                    let req: PluginRequest = parse_json_object(arguments)?;
                    let parts = ctx
                        .request_context
                        .extensions
                        .get::<Parts>()
                        .cloned()
                        .unwrap_or_else(|| http::Request::new(()).into_parts().0);
                    let args: Vec<String> =
                        ["objectiveai".to_string(), "plugins".to_string(), plugin_name]
                            .into_iter()
                            .chain(req.args.into_iter())
                            .collect();
                    let buf = run_cli_and_collect(
                        &cli_config,
                        &parts,
                        args,
                        test_mode,
                        OutputMode::Plugin,
                    )
                    .await;
                    Ok(CallToolResult::success(vec![Content::text(buf)]))
                }
                .boxed()
            }));
        }
        for cli_tool in tools {
            if cli_tool.name == "ObjectiveAI" {
                continue;
            }
            let tool_name = cli_tool.name.clone();
            let cli_config_for_route = cli_config.clone();
            let tool = Tool::new(
                Cow::Owned(cli_tool.name.clone()),
                Cow::Owned(cli_tool.manifest.description.clone()),
                schema_for_type::<ToolRequest>(),
            );
            tool_router.add_route(ToolRoute::new_dyn(tool, move |ctx| {
                let cli_config = cli_config_for_route.clone();
                let tool_name = tool_name.clone();
                async move {
                    let arguments = ctx.arguments.unwrap_or_default();
                    let req: ToolRequest = parse_json_object(arguments)?;
                    let parts = ctx
                        .request_context
                        .extensions
                        .get::<Parts>()
                        .cloned()
                        .unwrap_or_else(|| http::Request::new(()).into_parts().0);
                    let args: Vec<String> =
                        ["objectiveai".to_string(), "tools".to_string(), tool_name.clone()]
                            .into_iter()
                            .chain(req.args.into_iter())
                            .collect();
                    let buf = run_cli_and_collect(
                        &cli_config,
                        &parts,
                        args,
                        test_mode,
                        OutputMode::Tool,
                    )
                    .await;
                    Ok(CallToolResult::success(vec![Content::text(buf)]))
                }
                .boxed()
            }));
        }
        Self {
            tool_router,
            cli_config,
            test_mode,
        }
    }

    #[tool(
        name = "ObjectiveAI",
        description = "Run an ObjectiveAI command."
    )]
    async fn objectiveai(
        &self,
        Parameters(req): Parameters<ObjectiveAiRequest>,
        Extension(parts): Extension<Parts>,
    ) -> String {
        let args: Vec<String> = std::iter::once("objectiveai".to_string())
            .chain(req.command)
            .collect();
        // Catch-all dispatches arbitrary `objectiveai …` commands.
        // Plugin mode preserves the most structure (full
        // NotificationValue rather than extracting ToolLine.line),
        // which is the safer default when the underlying command
        // is unknown.
        run_cli_and_collect(
            &self.cli_config,
            &parts,
            args,
            self.test_mode,
            OutputMode::Plugin,
        )
        .await
    }
}

/// Run the ObjectiveAI CLI in-process with `args`, collecting every
/// emitted `Output`, and format the result for the MCP response.
/// Applies the per-request `X-OBJECTIVEAI-AGENT-ID` header override
/// (clones the server-wide `cli_config` so concurrent requests stay
/// independent).
///
/// `mode` selects plugin vs. tool rendering — see [`crate::format`]
/// for the full dispatch table. When `test_mode` is `true`,
/// `agent_id` is stripped from rendered errors (notification
/// renderings drop it by construction). Production callers leave
/// it off so cross-process correlation stamps survive. See the
/// `TEST_MODE` env var in `run.rs`.
async fn run_cli_and_collect(
    cli_config: &Arc<objectiveai_cli::Config>,
    parts: &Parts,
    args: Vec<String>,
    test_mode: bool,
    mode: OutputMode,
) -> String {
    // Per-request: stamp agent_id + mcp_session_id from headers so
    // every cli invocation (and every tool subprocess the cli spawns
    // transitively) sees the values relevant to *this* request.
    // Clone-then-mutate-then-Arc so concurrent requests see
    // independent values.
    //
    // The MCP session id is read from `Mcp-Session-Id` (the rmcp
    // transport's standard header). When the upstream MCP client
    // doesn't actually manage sessions, fall back to the per-agent
    // lineage id from `X-OBJECTIVEAI-AGENT-ID` — stable per agent and
    // unique across agents, which is exactly what session-keyed tool
    // state (e.g. per-session counters) wants.
    let mut cfg = (**cli_config).clone();
    let header_agent_id = parts
        .headers
        .get("X-OBJECTIVEAI-AGENT-ID")
        .and_then(|h| h.to_str().ok());
    if let Some(agent_id) = header_agent_id {
        cfg.agent_id = Some(agent_id.to_string());
    }
    let header_session_id = parts
        .headers
        .get(objectiveai_sdk::mcp::MCP_SESSION_ID_HEADER)
        .and_then(|h| h.to_str().ok());
    cfg.mcp_session_id = match header_session_id {
        Some(s) => Some(s.to_string()),
        None => header_agent_id.map(str::to_string),
    };
    let cli_config: Arc<objectiveai_cli::Config> = Arc::new(cfg);

    let collected = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    // Per-request handle: stamp the agent_id from the per-request
    // cli_config so every notification/error the cli emits during
    // this Run carries `X-OBJECTIVEAI-AGENT-ID`. The Collect
    // destination mirrors the same agent_id into the in-memory
    // Vec the runner reassembles below.
    let handle = objectiveai_sdk::cli::output::Handle {
        destination: objectiveai_sdk::cli::output::HandleDestination::Collect(collected.clone()),
        agent_id: cli_config.agent_id.clone(),
    };
    let _ = objectiveai_cli::run(args, &cli_config, handle).await;

    let outputs = collected.lock().await;
    format_outputs(mode, &outputs, test_mode)
}

#[tool_handler]
impl ServerHandler for ObjectiveAiMcpCli {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2025_06_18,
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "oai".into(),
                title: None,
                version: env!("CARGO_PKG_VERSION").into(),
                description: None,
                icons: None,
                website_url: None,
            },
            instructions: None,
        }
    }
}
