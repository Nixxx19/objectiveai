use std::sync::Arc;

use indexmap::IndexMap;
use serde_json::{Value, json};

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

const ACCEPT: &str = "application/json, text/event-stream";

fn init_params() -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": { "name": "test", "version": "0.1.0" }
        }
    })
}

/// Parse a response that may be JSON or SSE (text/event-stream with `data:` lines).
async fn parse_response(resp: reqwest::Response) -> Value {
    let body = resp.text().await.unwrap();
    if let Ok(v) = serde_json::from_str::<Value>(&body) {
        return v;
    }
    let data: String = body
        .lines()
        .filter_map(|l| l.strip_prefix("data: "))
        .collect::<Vec<_>>()
        .join("");
    serde_json::from_str(&data)
        .unwrap_or_else(|e| panic!("failed to parse response as JSON or SSE: {e}\nbody: {body}"))
}

/// Bring up a fresh, isolated multi-tenant server with one tenant
/// holding `tools`. Returns `(spawner, session, url)`. Tests must keep
/// the spawner+session alive for the duration of the test (drop frees
/// the tenant; spawner drop frees the server).
async fn make_server(
    tools: Vec<InventionTool>,
) -> (Arc<InventionServerSpawner>, InventionSession, String) {
    let spawner = Arc::new(InventionServerSpawner::new());
    let handle = spawner.get().await.expect("spawn invention server");
    let session = handle.register(tools);
    let url = session.url();
    (spawner, session, url)
}

/// Send initialize + notifications/initialized, return the rmcp session id header.
async fn init_session(
    client: &reqwest::Client,
    base_url: &str,
    tenant_id: &str,
) -> String {
    let resp = client
        .post(base_url)
        .header("Accept", ACCEPT)
        .header(TENANT_HEADER, tenant_id)
        .json(&init_params())
        .send()
        .await
        .unwrap();
    let session_id = resp
        .headers()
        .get("mcp-session-id")
        .map(|v| v.to_str().unwrap().to_string())
        .unwrap_or_default();
    let _body = parse_response(resp).await;

    client
        .post(base_url)
        .header("Accept", ACCEPT)
        .header("mcp-session-id", &session_id)
        .header(TENANT_HEADER, tenant_id)
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        }))
        .send()
        .await
        .unwrap();

    session_id
}

/// Send a JSON-RPC request as a particular tenant + rmcp session.
async fn rpc(
    client: &reqwest::Client,
    url: &str,
    session_id: &str,
    tenant_id: &str,
    body: Value,
) -> Value {
    let resp = client
        .post(url)
        .header("Accept", ACCEPT)
        .header("mcp-session-id", session_id)
        .header(TENANT_HEADER, tenant_id)
        .json(&body)
        .send()
        .await
        .unwrap();
    parse_response(resp).await
}

#[tokio::test]
async fn test_initialize() {
    let (_spawner, session, url) = make_server(vec![]).await;
    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .header("Accept", ACCEPT)
        .header(TENANT_HEADER, session.id())
        .json(&init_params())
        .send()
        .await
        .unwrap();
    let resp = parse_response(resp).await;

    assert!(resp["result"]["protocolVersion"].is_string());
    assert!(resp["result"]["serverInfo"].is_object());
}

#[tokio::test]
async fn test_notifications_initialized() {
    let (_spawner, session, url) = make_server(vec![]).await;
    let client = reqwest::Client::new();
    let session_id = init_session(&client, &url, session.id()).await;
    assert!(!session_id.is_empty());
}

#[tokio::test]
async fn test_tools_list() {
    let (_spawner, session, url) = make_server(vec![echo_tool()]).await;
    let client = reqwest::Client::new();
    let mcp_session = init_session(&client, &url, session.id()).await;

    let resp = rpc(
        &client,
        &url,
        &mcp_session,
        session.id(),
        json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        }),
    )
    .await;

    let tools = resp["result"]["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0]["name"], "echo");
    assert_eq!(tools[0]["description"], "Echoes back the input");
}

#[tokio::test]
async fn test_tools_call_success() {
    let (_spawner, session, url) = make_server(vec![echo_tool()]).await;
    let client = reqwest::Client::new();
    let mcp_session = init_session(&client, &url, session.id()).await;

    let resp = rpc(
        &client,
        &url,
        &mcp_session,
        session.id(),
        json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "echo",
                "arguments": { "text": "hello world" }
            }
        }),
    )
    .await;

    assert_eq!(resp["result"]["isError"], false);
    assert_eq!(resp["result"]["content"][0]["type"], "text");
    assert_eq!(resp["result"]["content"][0]["text"], "hello world");
}

#[tokio::test]
async fn test_tools_call_error() {
    let (_spawner, session, url) = make_server(vec![failing_tool()]).await;
    let client = reqwest::Client::new();
    let mcp_session = init_session(&client, &url, session.id()).await;

    let resp = rpc(
        &client,
        &url,
        &mcp_session,
        session.id(),
        json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "tools/call",
            "params": {
                "name": "fail",
                "arguments": {}
            }
        }),
    )
    .await;

    assert_eq!(resp["result"]["isError"], true);
    assert_eq!(
        resp["result"]["content"][0]["text"],
        "something went wrong"
    );
}

#[tokio::test]
async fn test_tools_call_not_found() {
    let (_spawner, session, url) = make_server(vec![]).await;
    let client = reqwest::Client::new();
    let mcp_session = init_session(&client, &url, session.id()).await;

    let resp = rpc(
        &client,
        &url,
        &mcp_session,
        session.id(),
        json!({
            "jsonrpc": "2.0",
            "id": 5,
            "method": "tools/call",
            "params": {
                "name": "nonexistent",
                "arguments": {}
            }
        }),
    )
    .await;

    assert!(resp["error"].is_object());
    assert!(resp["error"]["code"].as_i64().unwrap() < 0);
}

#[tokio::test]
async fn test_url() {
    let (_spawner, _session, url) = make_server(vec![]).await;
    assert!(url.starts_with("http://127.0.0.1:"));
    assert!(url.ends_with("/mcp"));
}

#[tokio::test]
async fn test_unknown_method() {
    let (_spawner, session, url) = make_server(vec![]).await;
    let client = reqwest::Client::new();
    let mcp_session = init_session(&client, &url, session.id()).await;

    let resp = rpc(
        &client,
        &url,
        &mcp_session,
        session.id(),
        json!({
            "jsonrpc": "2.0",
            "id": 6,
            "method": "unknown/method",
            "params": {}
        }),
    )
    .await;

    assert!(resp["error"].is_object());
    assert!(resp["error"]["code"].as_i64().unwrap() < 0);
}

/// Two tenants on the same shared server hold disjoint tool sets and
/// don't see each other's tools.
#[tokio::test]
async fn test_multi_tenant_isolation() {
    let spawner = Arc::new(InventionServerSpawner::new());
    let handle = spawner.get().await.expect("spawn invention server");
    let session_a = handle.register(vec![echo_tool()]);
    let session_b = handle.register(vec![failing_tool()]);
    let url = session_a.url();
    assert_eq!(url, session_b.url(), "tenants share one URL");

    let client = reqwest::Client::new();
    let mcp_session_a = init_session(&client, &url, session_a.id()).await;
    let mcp_session_b = init_session(&client, &url, session_b.id()).await;

    let list_a = rpc(
        &client, &url, &mcp_session_a, session_a.id(),
        json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}),
    )
    .await;
    let names_a: Vec<&str> = list_a["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert_eq!(names_a, vec!["echo"]);

    let list_b = rpc(
        &client, &url, &mcp_session_b, session_b.id(),
        json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}),
    )
    .await;
    let names_b: Vec<&str> = list_b["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert_eq!(names_b, vec!["fail"]);
}
