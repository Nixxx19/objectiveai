//! MCP client for creating connections to MCP servers.

use std::time::Duration;

use indexmap::IndexMap;


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
    pub user_agent: String,
    /// X-Title header value.
    pub x_title: String,
    /// Referer header value.
    pub http_referer: String,
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
        user_agent: String,
        x_title: String,
        http_referer: String,
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
            http_referer,
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

    /// Returns the innate headers this client stamps on every request
    /// (`User-Agent`, `X-Title`, `Referer`, `HTTP-Referer`). Useful for
    /// callers that need to forward the same identity through a proxy
    /// without hardcoding which fields the proxy expects.
    pub fn headers(&self) -> IndexMap<String, String> {
        let mut headers = IndexMap::new();
        headers.insert("User-Agent".to_string(), self.user_agent.clone());
        headers.insert("X-Title".to_string(), self.x_title.clone());
        headers.insert("Referer".to_string(), self.http_referer.clone());
        headers.insert("HTTP-Referer".to_string(), self.http_referer.clone());
        headers
    }

    /// Connects to an MCP server using the Streamable HTTP transport.
    ///
    /// Sends an `initialize` JSON-RPC request to the server and extracts
    /// the `Mcp-Session-Id` from the response. Returns a [`Connection`]
    /// that can be used to list/call tools and list/read resources.
    ///
    /// `extra_headers` are forwarded on every request this connection
    /// makes to the upstream — both the initial `initialize` POST and
    /// every subsequent RPC. They are applied *after* the fixed headers
    /// so callers can't accidentally clobber `Mcp-Session-Id`,
    /// `Content-Type`, etc.
    ///
    /// ## SSE handoff
    ///
    /// `Accept` is `text/event-stream, application/json` — stream first
    /// — so the server is encouraged to keep the underlying connection
    /// open. If the response comes back as SSE we read the initialize
    /// event off the stream and hand the *still-open* line reader to the
    /// returned [`Connection`]'s list-changed listener. The listener
    /// starts reading from that pre-opened stream immediately, which
    /// closes the race where a peer (e.g. an in-process rmcp upstream)
    /// would broadcast `notifications/tools/list_changed` before our
    /// listener had managed to open its own GET `/` SSE.
    ///
    /// If the response is unary JSON and the server advertises either
    /// `tools.list_changed` or `resources.list_changed`, we proactively
    /// open a GET `/` SSE stream *before returning* and hand it to the
    /// listener for the same reason. If neither capability is set, no
    /// listener is needed and we return without touching SSE.
    pub async fn connect(
        &self,
        url: String,
        authorization: Option<String>,
        session_id: Option<String>,
        extra_headers: IndexMap<String, String>,
    ) -> Result<super::Connection, super::Error> {
        if url == "mock" {
            return Ok(super::Connection::new_mock(url));
        }

        let init_request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-06-18",
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
            .header("Accept", "text/event-stream, application/json")
            .json(&init_request);

        if let Some(sid) = &session_id {
            request = request.header("Mcp-Session-Id", sid);
        }
        if let Some(auth) = &authorization {
            request = request.header("Authorization", auth);
        }
        request = request.header("User-Agent", &self.user_agent);
        request = request.header("X-Title", &self.x_title);
        request = request.header("Referer", &self.http_referer);
        request = request.header("HTTP-Referer", &self.http_referer);
        for (name, value) in &extra_headers {
            request = request.header(name, value);
        }

        let response = super::send_with_transient_retry(request)
            .await
            .map_err(|source| super::Error::Connection {
                url: url.clone(),
                source,
            })?;

        if !response.status().is_success() {
            let code = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(super::Error::BadStatus {
                url: url.clone(),
                code,
                body,
            });
        }

        // Extract session ID from response header.
        let session_id = match response
            .headers()
            .get("Mcp-Session-Id")
            .and_then(|v| v.to_str().ok())
            .map(String::from)
        {
            Some(s) => s,
            None => {
                let body = response.text().await.unwrap_or_default();
                return Err(super::Error::NoSessionId {
                    url: url.clone(),
                    body: body.chars().take(800).collect(),
                });
            }
        };

        // Did the server return SSE or unary JSON? rmcp's
        // `StreamableHttpService` always returns SSE; many other servers
        // reply with bare JSON.
        let is_sse = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(|v| v.starts_with("text/event-stream"))
            .unwrap_or(false);

        // Parse the initialize response. SSE path consumes one event
        // from the stream and keeps the rest of the stream alive for
        // the listener; unary path consumes the whole body.
        let (initialize_result, mut initial_sse_lines) = if is_sse {
            let mut lines = super::lines_from_response(response);
            let rpc_response: super::JsonRpcResponse<
                super::initialize_result::InitializeResult,
            > = super::read_next_sse_event(&url, &mut lines).await?;
            let result = match rpc_response {
                super::JsonRpcResponse::Success { result, .. } => result,
                super::JsonRpcResponse::Error { error, .. } => {
                    return Err(super::Error::JsonRpc {
                        url: url.clone(),
                        code: error.code,
                        message: error.message,
                        data: error.data,
                    });
                }
            };
            (result, Some(lines))
        } else {
            let rpc_response: super::JsonRpcResponse<
                super::initialize_result::InitializeResult,
            > = super::parse_streamable_http_response(&url, response).await?;
            let result = match rpc_response {
                super::JsonRpcResponse::Success { result, .. } => result,
                super::JsonRpcResponse::Error { error, .. } => {
                    return Err(super::Error::JsonRpc {
                        url: url.clone(),
                        code: error.code,
                        message: error.message,
                        data: error.data,
                    });
                }
            };
            (result, None)
        };

        // Whether we need a notification SSE channel at all.
        let needs_sse = initialize_result
            .capabilities
            .tools
            .as_ref()
            .and_then(|t| t.list_changed)
            .unwrap_or(false)
            || initialize_result
                .capabilities
                .resources
                .as_ref()
                .and_then(|r| r.list_changed)
                .unwrap_or(false);

        // Send `notifications/initialized` BEFORE any other request.
        // rmcp's per-session worker is in `expect_notification("initialized")`
        // at this point — anything else (a `tools/list`, an opportunistic
        // GET `/`) that lands during that window pushes a non-notification
        // through the worker, makes `serve_server_with_ct_inner` return
        // `Err(ExpectedInitializedNotification(...))`, drops the
        // WorkerTransport, cancels the worker via its drop_guard, and
        // tears the whole session down. Every later POST then 500s with
        // "Session service terminated."
        //
        // We don't have a `Connection` yet — building one would spawn
        // `refresh_tools` / `refresh_resources` background tasks that
        // race with this notification, which is exactly the bug we're
        // avoiding. We therefore POST inline here.
        let init_notification_body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized",
            "params": {},
        });
        let mut notify_request = self
            .http_client
            .post(&url)
            .timeout(self.call_timeout)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .header("Mcp-Session-Id", &session_id);
        if let Some(auth) = &authorization {
            notify_request = notify_request.header("Authorization", auth);
        }
        notify_request = notify_request
            .header("User-Agent", &self.user_agent)
            .header("X-Title", &self.x_title)
            .header("Referer", &self.http_referer)
            .header("HTTP-Referer", &self.http_referer);
        for (name, value) in &extra_headers {
            notify_request = notify_request.header(name, value);
        }
        let notify_response = super::send_with_transient_retry(
            notify_request.json(&init_notification_body),
        )
        .await
        .map_err(|source| super::Error::Request {
            url: url.clone(),
            source,
        })?;
        if !notify_response.status().is_success() {
            let code = notify_response.status();
            let body = notify_response.text().await.unwrap_or_default();
            return Err(super::Error::BadStatus { url, code, body });
        }

        // Now safe to drop the init-side SSE stream; rmcp's session
        // worker is past the init handshake and we have no further use
        // for it (notifications come on the GET / stream below).
        drop(initial_sse_lines.take());

        // Now that the server's session worker is past the
        // `expect_notification` gate, it's safe to open the proactive
        // GET `/` SSE stream the listener will read
        // `notifications/{tools,resources}/list_changed` from. Capability
        // inspection only gates whether we open this stream; the
        // `Connection` itself is naive about capabilities.
        let initial_sse_lines: Option<super::LinesStream> = if needs_sse {
            let mut get_request = self
                .http_client
                .get(&url)
                .timeout(self.connect_timeout)
                .header("Accept", "text/event-stream")
                .header("Mcp-Session-Id", &session_id);
            if let Some(auth) = &authorization {
                get_request = get_request.header("Authorization", auth);
            }
            get_request = get_request
                .header("User-Agent", &self.user_agent)
                .header("X-Title", &self.x_title)
                .header("Referer", &self.http_referer)
                .header("HTTP-Referer", &self.http_referer);
            for (name, value) in &extra_headers {
                get_request = get_request.header(name, value);
            }
            let get_response = super::send_with_transient_retry(get_request)
                .await
                .map_err(|source| super::Error::Connection {
                    url: url.clone(),
                    source,
                })?;
            if !get_response.status().is_success() {
                let code = get_response.status();
                let body = get_response.text().await.unwrap_or_default();
                return Err(super::Error::BadStatus { url: url.clone(), code, body });
            }
            Some(super::lines_from_response(get_response))
        } else {
            None
        };

        // Construct the Connection at the very end. This is the only
        // place in `connect` where the listener task and the
        // `refresh_tools` / `refresh_resources` background tasks get
        // spawned — by now the upstream is fully past its init
        // handshake, so any of those POSTs land safely.
        let connection = super::Connection::new(
            self.http_client.clone(),
            url,
            session_id,
            authorization,
            self.user_agent.clone(),
            self.x_title.clone(),
            self.http_referer.clone(),
            extra_headers,
            self.backoff_current_interval,
            self.backoff_initial_interval,
            self.backoff_randomization_factor,
            self.backoff_multiplier,
            self.backoff_max_interval,
            self.backoff_max_elapsed_time,
            self.call_timeout,
            initialize_result,
            initial_sse_lines,
        )
        .await;

        Ok(connection)
    }
}
