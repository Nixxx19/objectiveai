//! MCP connection for communicating with an MCP server.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::sync::RwLock;

/// An active connection to an MCP server using the Streamable HTTP transport.
///
/// Created by [`Client::connect`](super::Client::connect). Provides methods
/// for listing/calling tools and listing/reading resources. All requests
/// include the `Mcp-Session-Id` header for session continuity.
///
/// On creation, background tasks are spawned to paginate through all tools
/// and resources. The write lock is held for the entire duration of
/// pagination, so readers block until all pages have been fetched.
#[derive(Debug)]
pub struct Connection {
    pub http_client: reqwest::Client,
    pub url: String,
    pub session_id: String,
    pub authorization: Option<String>,
    pub user_agent: Option<String>,
    pub x_title: Option<String>,
    pub referer: Option<String>,

    pub backoff_current_interval: Duration,
    pub backoff_initial_interval: Duration,
    pub backoff_randomization_factor: f64,
    pub backoff_multiplier: f64,
    pub backoff_max_interval: Duration,
    pub backoff_max_elapsed_time: Duration,
    pub call_timeout: Duration,

    /// The server's capabilities and info from the initialize response.
    pub initialize_result: super::initialize_result::InitializeResult,

    /// Auto-incrementing request ID (starts at 2; 1 was used for initialize).
    next_id: AtomicU64,

    /// All tools from the server, populated by background pagination.
    tools: RwLock<Result<Arc<Vec<super::tool::Tool>>, Arc<super::Error>>>,
    /// All resources from the server, populated by background pagination.
    resources:
        RwLock<Result<Arc<Vec<super::resource::Resource>>, Arc<super::Error>>>,
}

impl Connection {
    /// Creates a minimal connection for unit testing.
    /// Only the `url` field is meaningful; other fields are defaulted.
    #[cfg(test)]
    pub(crate) fn new_for_test(url: String) -> Arc<Self> {
        Arc::new(Self {
            http_client: reqwest::Client::new(),
            url,
            session_id: String::new(),
            authorization: None,
            user_agent: None,
            x_title: None,
            referer: None,
            backoff_current_interval: Duration::from_millis(500),
            backoff_initial_interval: Duration::from_millis(500),
            backoff_randomization_factor: 0.5,
            backoff_multiplier: 1.5,
            backoff_max_interval: Duration::from_secs(60),
            backoff_max_elapsed_time: Duration::from_secs(900),
            call_timeout: Duration::from_secs(30),
            initialize_result: super::initialize_result::InitializeResult {
                protocol_version: "2025-03-26".into(),
                capabilities:
                    super::initialize_result::ServerCapabilities {
                        experimental: None,
                        logging: None,
                        completions: None,
                        prompts: None,
                        resources: None,
                        tools: None,
                        tasks: None,
                    },
                server_info: super::initialize_result::Implementation {
                    name: "test".into(),
                    title: None,
                    version: "0.0.0".into(),
                    website_url: None,
                    description: None,
                    icons: None,
                },
                instructions: None,
                _meta: None,
            },
            next_id: AtomicU64::new(2),
            tools: RwLock::new(Ok(Arc::new(Vec::new()))),
            resources: RwLock::new(Ok(Arc::new(Vec::new()))),
        })
    }

    /// Creates a new connection and spawns background tasks to paginate
    /// all tools and resources. Called internally by
    /// [`Client::connect`](super::Client::connect).
    pub(super) fn new(
        http_client: reqwest::Client,
        url: String,
        session_id: String,
        authorization: Option<String>,
        user_agent: Option<String>,
        x_title: Option<String>,
        referer: Option<String>,
        backoff_current_interval: Duration,
        backoff_initial_interval: Duration,
        backoff_randomization_factor: f64,
        backoff_multiplier: f64,
        backoff_max_interval: Duration,
        backoff_max_elapsed_time: Duration,
        call_timeout: Duration,
        initialize_result: super::initialize_result::InitializeResult,
    ) -> Arc<Self> {
        let conn = Arc::new(Self {
            http_client,
            url,
            session_id,
            authorization,
            user_agent,
            x_title,
            referer,
            backoff_current_interval,
            backoff_initial_interval,
            backoff_randomization_factor,
            backoff_multiplier,
            backoff_max_interval,
            backoff_max_elapsed_time,
            call_timeout,
            initialize_result,
            next_id: AtomicU64::new(2),
            tools: RwLock::new(Ok(Arc::new(Vec::new()))),
            resources: RwLock::new(Ok(Arc::new(Vec::new()))),
        });

        // Spawn background tool lister if the server supports tools.
        if conn.initialize_result.capabilities.tools.is_some() {
            let conn = Arc::clone(&conn);
            tokio::spawn(async move {
                conn.refresh_tools().await;
            });
        }

        // Spawn background resource lister if the server supports resources.
        if conn.initialize_result.capabilities.resources.is_some() {
            let conn = Arc::clone(&conn);
            tokio::spawn(async move {
                conn.refresh_resources().await;
            });
        }

        // Spawn listener for list_changed notifications if supported.
        {
            let tools_list_changed = conn
                .initialize_result
                .capabilities
                .tools
                .and_then(|t| t.list_changed)
                .unwrap_or(false);
            let resources_list_changed = conn
                .initialize_result
                .capabilities
                .resources
                .and_then(|r| r.list_changed)
                .unwrap_or(false);

            if tools_list_changed || resources_list_changed {
                let conn = Arc::clone(&conn);
                tokio::spawn(async move {
                    conn.listen_for_list_changes(
                        tools_list_changed,
                        resources_list_changed,
                    )
                    .await;
                });
            }
        }

        conn
    }

    /// Creates an exponential backoff configuration from the connection's fields.
    fn backoff(&self) -> backoff::ExponentialBackoff {
        backoff::ExponentialBackoff {
            current_interval: self.backoff_current_interval,
            initial_interval: self.backoff_initial_interval,
            randomization_factor: self.backoff_randomization_factor,
            multiplier: self.backoff_multiplier,
            max_interval: self.backoff_max_interval,
            start_time: std::time::Instant::now(),
            max_elapsed_time: Some(self.backoff_max_elapsed_time),
            clock: backoff::SystemClock::default(),
        }
    }

    /// Builds a POST request with all required headers and the call timeout.
    fn post(&self) -> reqwest::RequestBuilder {
        let mut request = self
            .http_client
            .post(&self.url)
            .timeout(self.call_timeout)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .header("Mcp-Session-Id", &self.session_id);

        if let Some(auth) = &self.authorization {
            request = request.header("Authorization", auth);
        }
        if let Some(ua) = &self.user_agent {
            request = request.header("User-Agent", ua);
        }
        if let Some(title) = &self.x_title {
            request = request.header("X-Title", title);
        }
        if let Some(referer) = &self.referer {
            request = request.header("Referer", referer);
        }
        request
    }

    /// Sends a JSON-RPC request with exponential backoff retries.
    ///
    /// Network errors and non-success HTTP status codes are retried.
    /// Session expiration (404) and JSON-RPC errors are permanent failures.
    async fn rpc<P: serde::Serialize, R: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        params: &P,
    ) -> Result<R, super::Error> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });

        backoff::future::retry(self.backoff(), || async {
            let response =
                self.post().json(&body).send().await.map_err(|e| {
                    backoff::Error::transient(super::Error::Request(e))
                })?;

            if response.status() == reqwest::StatusCode::NOT_FOUND {
                return Err(backoff::Error::permanent(
                    super::Error::SessionExpired,
                ));
            }
            if !response.status().is_success() {
                let code = response.status();
                let body = response.text().await.unwrap_or_default();
                return Err(backoff::Error::transient(
                    super::Error::BadStatus { code, body },
                ));
            }

            let rpc_response: JsonRpcResponse<R> =
                response.json().await.map_err(|e| {
                    backoff::Error::transient(super::Error::Request(e))
                })?;

            match rpc_response {
                JsonRpcResponse::Success { result, .. } => Ok(result),
                JsonRpcResponse::Error { error, .. } => {
                    Err(backoff::Error::permanent(super::Error::JsonRpc {
                        code: error.code,
                        message: error.message,
                        data: error.data,
                    }))
                }
            }
        })
        .await
    }

    /// Sends a JSON-RPC notification (no response expected, no retries).
    pub(super) async fn notify<P: serde::Serialize>(
        &self,
        method: &str,
        params: &P,
    ) -> Result<(), super::Error> {
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
        });

        let response = self
            .post()
            .json(&body)
            .send()
            .await
            .map_err(super::Error::Request)?;

        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(super::Error::SessionExpired);
        }
        if !response.status().is_success() {
            let code = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(super::Error::BadStatus { code, body });
        }

        Ok(())
    }

    /// Returns a key identifying this connection for tool namespacing.
    pub fn tool_key(&self) -> String {
        format!("{}-{}", self.initialize_result.server_info.name, self.url)
    }

    /// Returns the session ID for this connection.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Sends a `tools/list` RPC call for a single page.
    async fn rpc_list_tools(
        &self,
        cursor: Option<&str>,
    ) -> Result<super::tool::ListToolsResult, super::Error> {
        self.rpc(
            "tools/list",
            &super::tool::ListToolsRequest {
                cursor: cursor.map(String::from),
            },
        )
        .await
    }

    /// Returns all tools from the server.
    ///
    /// Blocks until background pagination completes, then returns a
    /// cheap `Arc` clone of the result.
    pub async fn list_tools(
        &self,
    ) -> Result<Arc<Vec<super::tool::Tool>>, Arc<super::Error>> {
        self.tools.read().await.clone()
    }

    /// Calls a tool on the MCP server.
    pub async fn call_tool(
        &self,
        params: &super::tool::CallToolRequestParams,
    ) -> Result<super::tool::CallToolResult, super::Error> {
        self.rpc("tools/call", params).await
    }

    /// Sends a `resources/list` RPC call for a single page.
    async fn rpc_list_resources(
        &self,
        cursor: Option<&str>,
    ) -> Result<super::resource::ListResourcesResult, super::Error> {
        self.rpc(
            "resources/list",
            &super::resource::ListResourcesRequest {
                cursor: cursor.map(String::from),
            },
        )
        .await
    }

    /// Returns all resources from the server.
    ///
    /// Blocks until background pagination completes, then returns a
    /// cheap `Arc` clone of the result.
    pub async fn list_resources(
        &self,
    ) -> Result<Arc<Vec<super::resource::Resource>>, Arc<super::Error>> {
        self.resources.read().await.clone()
    }

    /// Reads a resource from the MCP server.
    pub async fn read_resource(
        &self,
        uri: &str,
    ) -> Result<super::resource::ReadResourceResult, super::Error> {
        self.rpc(
            "resources/read",
            &super::resource::ReadResourceRequestParams {
                uri: uri.to_string(),
            },
        )
        .await
    }

    /// Re-fetches all tools from the server, replacing the cached list.
    async fn refresh_tools(&self) {
        let mut guard = self.tools.write().await;
        let mut all_tools = Vec::new();
        let mut cursor: Option<String> = None;
        let result = loop {
            match self.rpc_list_tools(cursor.as_deref()).await {
                Ok(page) => {
                    all_tools.extend(page.tools);
                    cursor = page.next_cursor;
                    if cursor.is_none() {
                        break Ok(Arc::new(all_tools));
                    }
                }
                Err(e) => break Err(Arc::new(e)),
            }
        };
        *guard = result;
    }

    /// Re-fetches all resources from the server, replacing the cached list.
    async fn refresh_resources(&self) {
        let mut guard = self.resources.write().await;
        let mut all_resources = Vec::new();
        let mut cursor: Option<String> = None;
        let result = loop {
            match self.rpc_list_resources(cursor.as_deref()).await {
                Ok(page) => {
                    all_resources.extend(page.resources);
                    cursor = page.next_cursor;
                    if cursor.is_none() {
                        break Ok(Arc::new(all_resources));
                    }
                }
                Err(e) => break Err(Arc::new(e)),
            }
        };
        *guard = result;
    }

    /// Builds a GET request to the MCP endpoint for receiving server
    /// notifications via SSE.
    fn get(&self) -> reqwest::RequestBuilder {
        let mut request = self
            .http_client
            .get(&self.url)
            .header("Accept", "text/event-stream")
            .header("Mcp-Session-Id", &self.session_id);

        if let Some(auth) = &self.authorization {
            request = request.header("Authorization", auth);
        }
        if let Some(ua) = &self.user_agent {
            request = request.header("User-Agent", ua);
        }
        if let Some(title) = &self.x_title {
            request = request.header("X-Title", title);
        }
        if let Some(referer) = &self.referer {
            request = request.header("Referer", referer);
        }
        request
    }

    /// Opens a GET SSE stream to the MCP endpoint and listens for
    /// `notifications/tools/list_changed` and
    /// `notifications/resources/list_changed`. On each notification,
    /// write-locks and re-fetches the full list. Reconnects on
    /// disconnection with a brief delay.
    async fn listen_for_list_changes(
        &self,
        tools: bool,
        resources: bool,
    ) {
        use futures_util::TryStreamExt;
        use tokio::io::AsyncBufReadExt;
        use tokio_util::io::StreamReader;

        loop {
            let response = match self.get().send().await {
                Ok(r) if r.status().is_success() => r,
                _ => {
                    tokio::time::sleep(self.backoff_initial_interval).await;
                    continue;
                }
            };

            let stream = response
                .bytes_stream()
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e));
            let reader = StreamReader::new(stream);
            let mut lines = reader.lines();

            while let Ok(Some(line)) = lines.next_line().await {
                // SSE data lines start with "data: ".
                let data = match line.strip_prefix("data: ") {
                    Some(d) => d,
                    None => continue,
                };

                let method = match serde_json::from_str::<JsonRpcNotification>(data) {
                    Ok(n) => n.method,
                    Err(_) => continue,
                };

                match method.as_str() {
                    "notifications/tools/list_changed" if tools => {
                        self.refresh_tools().await;
                    }
                    "notifications/resources/list_changed" if resources => {
                        self.refresh_resources().await;
                    }
                    _ => {}
                }
            }

            // Stream ended — reconnect after a brief delay.
            tokio::time::sleep(self.backoff_initial_interval).await;
        }
    }
}

/// JSON-RPC 2.0 response envelope.
#[derive(serde::Deserialize)]
#[serde(untagged)]
pub(super) enum JsonRpcResponse<T> {
    Success {
        #[allow(dead_code)]
        jsonrpc: String,
        #[allow(dead_code)]
        id: serde_json::Value,
        result: T,
    },
    Error {
        #[allow(dead_code)]
        jsonrpc: String,
        #[allow(dead_code)]
        id: serde_json::Value,
        error: JsonRpcError,
    },
}

/// JSON-RPC 2.0 error object.
#[derive(serde::Deserialize)]
pub(super) struct JsonRpcError {
    pub code: i64,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// JSON-RPC 2.0 notification (no `id` field).
#[derive(serde::Deserialize)]
struct JsonRpcNotification {
    #[allow(dead_code)]
    jsonrpc: String,
    method: String,
}
