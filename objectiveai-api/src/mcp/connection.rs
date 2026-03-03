//! MCP connection for communicating with an MCP server.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// An active connection to an MCP server using the Streamable HTTP transport.
///
/// Created by [`Client::connect`](super::Client::connect). Provides methods
/// for listing/calling tools and listing/reading resources. All requests
/// include the `Mcp-Session-Id` header for session continuity.
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

    /// Auto-incrementing request ID (starts at 2; 1 was used for initialize).
    pub next_id: AtomicU64,
}

impl Connection {
    /// Creates a new connection. Called internally by [`Client::connect`](super::Client::connect).
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
    ) -> Self {
        Self {
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
            next_id: AtomicU64::new(2),
        }
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

    /// Returns the session ID for this connection.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Lists the tools available on the MCP server.
    pub async fn list_tools(
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

    /// Calls a tool on the MCP server.
    pub async fn call_tool(
        &self,
        params: &super::tool::CallToolRequestParams,
    ) -> Result<super::tool::CallToolResult, super::Error> {
        self.rpc("tools/call", params).await
    }

    /// Lists the resources available on the MCP server.
    pub async fn list_resources(
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
