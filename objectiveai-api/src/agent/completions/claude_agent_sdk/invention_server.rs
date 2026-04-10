use std::borrow::Cow;
use std::sync::Arc;

use rmcp::{
    ServerHandler,
    model::{
        CallToolRequestParams, CallToolResult, Content, ListToolsResult,
        ServerCapabilities, ServerInfo, Tool,
    },
    transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService,
        session::local::LocalSessionManager,
    },
    ServiceExt,
};
use serde_json::Value;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;

use super::mcp_server_config::{McpHttpServerConfig, McpHttpServerConfigType};
use objectiveai::functions::inventions::InventionTool;

pub struct InventionServer {
    pub(super) port: u16,
    _cancel: CancellationToken,
    server_handle: tokio::task::AbortHandle,
}

#[derive(Clone)]
struct InventionMcp {
    tools: Arc<Vec<InventionTool>>,
}

impl ServerHandler for InventionMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("ObjectiveAI invention tool server".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }

    fn list_tools(
        &self,
        _request: Option<rmcp::model::PaginatedRequestParams>,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, rmcp::ErrorData>> + Send + '_ {
        let tools: Vec<Tool> = self.tools.iter().map(|t| {
            let mut input_schema = serde_json::Map::new();
            input_schema.insert("type".to_string(), Value::String("object".to_string()));
            input_schema.insert("properties".to_string(), serde_json::to_value(&t.parameters).unwrap());
            Tool {
                name: Cow::Owned(t.name.to_string()),
                title: None,
                description: Some(Cow::Owned(t.description.to_string())),
                input_schema: Arc::new(input_schema),
                output_schema: None,
                annotations: None,
                execution: None,
                icons: None,
                meta: None,
            }
        }).collect();
        std::future::ready(Ok(ListToolsResult {
            tools,
            next_cursor: None,
            meta: None,
        }))
    }

    fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, rmcp::ErrorData>> + Send + '_ {
        let tools = self.tools.clone();
        async move {
            let name = request.name.as_ref();
            let arguments = request.arguments.map(|m| Value::Object(m)).unwrap_or(Value::Object(Default::default()));

            let tool = tools.iter().find(|t| t.name == name);
            match tool {
                Some(tool) => {
                    let result = (tool.call)(arguments).await;
                    match result {
                        Ok(text) => Ok(CallToolResult::success(vec![Content::text(text)])),
                        Err(text) => Ok(CallToolResult::error(vec![Content::text(text)])),
                    }
                }
                None => Err(rmcp::ErrorData::method_not_found::<rmcp::model::CallToolRequestMethod>()),
            }
        }
    }
}

impl InventionServer {
    pub async fn new(tools: Vec<InventionTool>) -> Self {
        let tools = Arc::new(tools);
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        let ct = CancellationToken::new();
        let ct_child = ct.child_token();

        let mcp = InventionMcp { tools };
        let service: StreamableHttpService<InventionMcp, LocalSessionManager> =
            StreamableHttpService::new(
                move || Ok(mcp.clone()),
                Default::default(),
                StreamableHttpServerConfig {
                    stateful_mode: true,
                    sse_keep_alive: None,
                    cancellation_token: ct_child,
                    ..Default::default()
                },
            );

        let router = axum::Router::new().nest_service("/mcp", service);

        let server_handle = tokio::spawn(async move {
            axum::serve(listener, router).await.ok();
        })
        .abort_handle();

        Self {
            port,
            _cancel: ct,
            server_handle,
        }
    }

    pub fn mcp_server_config(&self) -> McpHttpServerConfig {
        McpHttpServerConfig {
            r#type: McpHttpServerConfigType::Http,
            url: format!("http://127.0.0.1:{}/mcp", self.port),
            headers: None,
        }
    }
}

impl Drop for InventionServer {
    fn drop(&mut self) {
        self.server_handle.abort();
    }
}

#[cfg(test)]
#[path = "invention_server_tests.rs"]
mod tests;
