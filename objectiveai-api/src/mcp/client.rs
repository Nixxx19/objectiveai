//! MCP client for creating connections to MCP servers.

use std::sync::Arc;
use std::time::Duration;

/// Client for creating MCP connections.
///
/// Holds shared configuration (HTTP client, headers, backoff parameters)
/// and creates [`Connection`](super::Connection) instances via
/// [`connect`](Client::connect).
#[derive(Debug, Clone)]
pub struct Client {
    /// HTTP client for making requests.
    pub http_client: reqwest::Client,
    /// User-Agent header value.
    pub user_agent: Option<String>,
    /// X-Title header value.
    pub x_title: Option<String>,
    /// Referer header value.
    pub referer: Option<String>,
    /// Timeout for the initial connection (initialize request).
    pub connect_timeout: Duration,

    /// Current backoff interval for retry logic.
    pub backoff_current_interval: Duration,
    /// Initial backoff interval for retry logic.
    pub backoff_initial_interval: Duration,
    /// Randomization factor for backoff jitter.
    pub backoff_randomization_factor: f64,
    /// Multiplier for exponential backoff growth.
    pub backoff_multiplier: f64,
    /// Maximum backoff interval.
    pub backoff_max_interval: Duration,
    /// Maximum total time to spend on retries.
    pub backoff_max_elapsed_time: Duration,
    /// Timeout for individual RPC calls after connection is established.
    pub call_timeout: Duration,
}

impl Client {
    /// Creates a new MCP client.
    pub fn new(
        http_client: reqwest::Client,
        user_agent: Option<String>,
        x_title: Option<String>,
        referer: Option<String>,
        connect_timeout: Duration,
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
            user_agent,
            x_title,
            referer,
            connect_timeout,
            backoff_current_interval,
            backoff_initial_interval,
            backoff_randomization_factor,
            backoff_multiplier,
            backoff_max_interval,
            backoff_max_elapsed_time,
            call_timeout,
        }
    }

    /// Connects to an MCP server using the Streamable HTTP transport.
    ///
    /// Sends an `initialize` JSON-RPC request to the server and extracts
    /// the `Mcp-Session-Id` from the response. Returns a [`Connection`]
    /// that can be used to list/call tools and list/read resources.
    pub async fn connect(
        &self,
        url: String,
        authorization: Option<String>,
    ) -> Result<Arc<super::Connection>, super::Error> {
        let init_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-03-26",
                "capabilities": {},
                "clientInfo": {
                    "name": "objectiveai",
                    "version": env!("CARGO_PKG_VERSION"),
                }
            }
        });

        let mut request = self
            .http_client
            .post(&url)
            .timeout(self.connect_timeout)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .json(&init_request);

        if let Some(auth) = &authorization {
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

        let response =
            request.send().await.map_err(super::Error::Connection)?;

        if !response.status().is_success() {
            let code = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(super::Error::BadStatus { code, body });
        }

        // Extract session ID from response header.
        let session_id = response
            .headers()
            .get("Mcp-Session-Id")
            .and_then(|v| v.to_str().ok())
            .map(String::from)
            .ok_or(super::Error::NoSessionId)?;

        // Parse the initialize result to confirm success.
        let rpc_response: super::connection::JsonRpcResponse<
            serde_json::Value,
        > = response.json().await.map_err(super::Error::Request)?;

        if let super::connection::JsonRpcResponse::Error { error, .. } =
            rpc_response
        {
            return Err(super::Error::JsonRpc {
                code: error.code,
                message: error.message,
                data: error.data,
            });
        }

        let connection = Arc::new(super::Connection::new(
            self.http_client.clone(),
            url,
            session_id,
            authorization,
            self.user_agent.clone(),
            self.x_title.clone(),
            self.referer.clone(),
            self.backoff_current_interval,
            self.backoff_initial_interval,
            self.backoff_randomization_factor,
            self.backoff_multiplier,
            self.backoff_max_interval,
            self.backoff_max_elapsed_time,
            self.call_timeout,
        ));

        // Send the initialized notification.
        connection
            .notify("initialized", &serde_json::json!({}))
            .await?;

        Ok(connection)
    }
}
