use rmcp::{
    handler::server::router::tool::ToolRouter,
    handler::server::wrapper::Parameters,
    schemars, tool, tool_router,
};

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ObjectiveAiRequest {
    #[schemars(description = "The command arguments to pass to the ObjectiveAI CLI (e.g. [\"agents\", \"list\"] or [\"functions\", \"executions\", \"create\", \"--help\"])")]
    pub command: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct CliTools {
    pub tool_router: ToolRouter<Self>,
}

#[tool_router]
impl CliTools {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(
        name = "ObjectiveAI",
        description = "Run an ObjectiveAI CLI command. Supports all subcommands: agents, swarms, functions, api, schemas, viewer."
    )]
    async fn objectiveai(&self, Parameters(req): Parameters<ObjectiveAiRequest>) -> String {
        let args: Vec<String> = std::iter::once("objectiveai".to_string())
            .chain(req.command)
            .collect();

        match objectiveai_cli::run(args).await {
            Ok(output) => output,
            Err(e) => format!("error: {e}"),
        }
    }
}
