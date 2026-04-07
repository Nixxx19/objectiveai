use std::sync::Arc;

use indexmap::IndexMap;
use serde_json::{json, Value};

use super::*;
use objectiveai::functions::inventions::InventionTool;

fn echo_tool() -> InventionTool {
    InventionTool {
        name: "echo",
        description: "Echoes back the input",
        parameters: {
            let mut m = IndexMap::new();
            m.insert(
                "text".to_string(),
                json!({ "type": "string", "description": "Text to echo" }),
            );
            m
        },
        call: Arc::new(|args| {
            Box::pin(async move {
                let text = args
                    .get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("(empty)");
                Ok(text.to_string())
            })
        }),
    }
}

fn failing_tool() -> InventionTool {
    InventionTool {
        name: "fail",
        description: "Always fails",
        parameters: IndexMap::new(),
        call: Arc::new(|_| Box::pin(async { Err("something went wrong".to_string()) })),
    }
}

#[tokio::test]
async fn test_initialize() {
    let server = InventionServer::new(vec![]).await;
    let client = reqwest::Client::new();
    let resp: Value = client
        .post(&format!("http://127.0.0.1:{}/mcp", server.port))
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(resp["result"]["protocolVersion"], "2025-03-26");
    assert_eq!(
        resp["result"]["serverInfo"]["name"],
        "objectiveai-invention"
    );
}

#[tokio::test]
async fn test_notifications_initialized() {
    let server = InventionServer::new(vec![]).await;
    let client = reqwest::Client::new();
    let resp = client
        .post(&format!("http://127.0.0.1:{}/mcp", server.port))
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status(), 202);
}

#[tokio::test]
async fn test_tools_list() {
    let server = InventionServer::new(vec![echo_tool()]).await;
    let client = reqwest::Client::new();
    let resp: Value = client
        .post(&format!("http://127.0.0.1:{}/mcp", server.port))
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let tools = resp["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0]["name"], "echo");
    assert_eq!(tools[0]["description"], "Echoes back the input");
    assert_eq!(tools[0]["inputSchema"]["type"], "object");
    assert!(tools[0]["inputSchema"]["properties"]["text"].is_object());
}

#[tokio::test]
async fn test_tools_call_success() {
    let server = InventionServer::new(vec![echo_tool()]).await;
    let client = reqwest::Client::new();
    let resp: Value = client
        .post(&format!("http://127.0.0.1:{}/mcp", server.port))
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "echo",
                "arguments": { "text": "hello world" }
            }
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(resp["result"]["isError"], false);
    assert_eq!(resp["result"]["content"][0]["type"], "text");
    assert_eq!(resp["result"]["content"][0]["text"], "hello world");
}

#[tokio::test]
async fn test_tools_call_error() {
    let server = InventionServer::new(vec![failing_tool()]).await;
    let client = reqwest::Client::new();
    let resp: Value = client
        .post(&format!("http://127.0.0.1:{}/mcp", server.port))
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "fail",
                "arguments": {}
            }
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(resp["result"]["isError"], true);
    assert_eq!(
        resp["result"]["content"][0]["text"],
        "something went wrong"
    );
}

#[tokio::test]
async fn test_tools_call_not_found() {
    let server = InventionServer::new(vec![]).await;
    let client = reqwest::Client::new();
    let resp: Value = client
        .post(&format!("http://127.0.0.1:{}/mcp", server.port))
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "tools/call",
            "params": {
                "name": "nonexistent",
                "arguments": {}
            }
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert!(resp["error"].is_object());
    assert_eq!(resp["error"]["code"], -32601);
}

#[tokio::test]
async fn test_mcp_server_config() {
    let server = InventionServer::new(vec![]).await;
    let config = server.mcp_server_config();
    assert_eq!(config.r#type, McpHttpServerConfigType::Http);
    assert_eq!(
        config.url,
        format!("http://127.0.0.1:{}/mcp", server.port)
    );
    assert!(config.headers.is_none());
}

#[tokio::test]
async fn test_unknown_method() {
    let server = InventionServer::new(vec![]).await;
    let client = reqwest::Client::new();
    let resp: Value = client
        .post(&format!("http://127.0.0.1:{}/mcp", server.port))
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 6,
            "method": "unknown/method",
            "params": {}
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert!(resp["error"].is_object());
    assert_eq!(resp["error"]["code"], -32601);
}
