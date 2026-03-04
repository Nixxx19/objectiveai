use std::sync::Arc;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::post;
use axum::{Json, Router};
use serde_json::{json, Value};
use tokio::net::TcpListener;

use super::mcp_server_config::{McpHttpServerConfig, McpHttpServerConfigType};
use objectiveai::functions::inventions::InventionTool;

pub struct InventionServer {
    pub(super) port: u16,
    server_handle: tokio::task::AbortHandle,
}

impl InventionServer {
    pub async fn new(tools: Vec<InventionTool>) -> Self {
        let tools = Arc::new(tools);
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        let router = Router::new()
            .route("/mcp", post(handle_jsonrpc))
            .with_state(tools);

        let server_handle = tokio::spawn(async move {
            axum::serve(listener, router).await.ok();
        })
        .abort_handle();

        Self {
            port,
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

pub(super) async fn handle_jsonrpc(
    axum::extract::State(tools): axum::extract::State<Arc<Vec<InventionTool>>>,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let method = body.get("method").and_then(|m| m.as_str()).unwrap_or("");
    let id = body.get("id").cloned();

    match method {
        "initialize" => Json(jsonrpc_response(
            id,
            json!({
                "protocolVersion": "2025-03-26",
                "capabilities": { "tools": {} },
                "serverInfo": {
                    "name": "objectiveai-invention",
                    "version": "0.1.0"
                }
            }),
        ))
        .into_response(),

        "notifications/initialized" => StatusCode::ACCEPTED.into_response(),

        "tools/list" => {
            let tool_list: Vec<Value> = tools
                .iter()
                .map(|t| {
                    json!({
                        "name": t.name,
                        "description": t.description,
                        "inputSchema": {
                            "type": "object",
                            "properties": t.parameters,
                        }
                    })
                })
                .collect();

            Json(jsonrpc_response(id, json!({ "tools": tool_list }))).into_response()
        }

        "tools/call" => {
            let params = body.get("params").cloned().unwrap_or(json!({}));
            let name = params
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("");
            let arguments = params
                .get("arguments")
                .cloned()
                .unwrap_or(json!({}));

            let tool = tools.iter().find(|t| t.name == name);

            match tool {
                Some(tool) => {
                    let result = (tool.call)(arguments).await;
                    match result {
                        Ok(text) => Json(jsonrpc_response(
                            id,
                            json!({
                                "content": [{ "type": "text", "text": text }],
                                "isError": false
                            }),
                        ))
                        .into_response(),
                        Err(text) => Json(jsonrpc_response(
                            id,
                            json!({
                                "content": [{ "type": "text", "text": text }],
                                "isError": true
                            }),
                        ))
                        .into_response(),
                    }
                }
                None => Json(jsonrpc_error(
                    id,
                    -32601,
                    &format!("tool not found: {name}"),
                ))
                .into_response(),
            }
        }

        _ => Json(jsonrpc_error(id, -32601, &format!("unknown method: {method}")))
            .into_response(),
    }
}

fn jsonrpc_response(id: Option<Value>, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

fn jsonrpc_error(id: Option<Value>, code: i32, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    })
}

#[cfg(test)]
#[path = "invention_server_tests.rs"]
mod tests;
