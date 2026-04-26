//! MCP Streamable-HTTP endpoints. POST handles JSON-RPC requests +
//! notifications + responses; GET serves the server-initiated SSE stream.

use std::convert::Infallible;
use std::time::Duration;

use axum::{
    extract::{Json, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{
        IntoResponse, Response,
        sse::{Event, KeepAlive, Sse},
    },
};
use futures::stream::{Stream, StreamExt};
use objectiveai::mcp::{
    JsonRpcError, JsonRpcRequest, JsonRpcResponse,
    initialize_result::{
        Implementation, InitializeResult, ResourcesCapability, ServerCapabilities,
        ToolsCapability,
    },
    resource::ReadResourceRequestParams,
    tool::CallToolRequestParams,
};
use tokio_stream::wrappers::BroadcastStream;

use crate::AppState;
use crate::sessions::{CallToolError, ReadResourceError, SessionManager};
use crate::upstream::{BadInit, connect_all, parse_init_headers};

/// MCP protocol version this proxy speaks.
const PROTOCOL_VERSION: &str = "2025-06-18";

/// JSON-RPC error codes we use.
const PARSE_ERROR: i64 = -32700;
const INVALID_REQUEST: i64 = -32600;
const METHOD_NOT_FOUND: i64 = -32601;
const INVALID_PARAMS: i64 = -32602;
const INTERNAL_ERROR: i64 = -32603;

/// Header the client sends to identify its session on every request after
/// `initialize`.
const SESSION_ID_HEADER: &str = "Mcp-Session-Id";

/// SSE keepalive cadence — picks something well under typical proxy /
/// load balancer idle timeouts.
const SSE_KEEP_ALIVE: Duration = Duration::from_secs(15);

// ---- POST handler ----------------------------------------------------------

/// POST `/`: receive a single JSON-RPC envelope, dispatch by method.
pub async fn handle_post(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<serde_json::Value>,
) -> Response {
    // Notifications and responses (no `id`) get 202 Accepted with no body.
    // The proxy doesn't yet act on either; this matches the spec's
    // requirement that the server accepts them.
    if body.get("id").is_none() {
        return StatusCode::ACCEPTED.into_response();
    }

    let request: JsonRpcRequest = match serde_json::from_value(body) {
        Ok(r) => r,
        Err(e) => return parse_error_response(format!("invalid JSON-RPC envelope: {e}")),
    };

    match request.method.as_str() {
        "initialize" => handle_initialize(&state, &headers, request).await,
        "ping" => handle_ping(request),
        "tools/list" => handle_tools_list(&state.sessions, &headers, request).await,
        "tools/call" => handle_tools_call(&state.sessions, &headers, request).await,
        "resources/list" => handle_resources_list(&state.sessions, &headers, request).await,
        "resources/read" => handle_resources_read(&state.sessions, &headers, request).await,
        other => method_not_found_response(request.id, other),
    }
}

// ---- GET handler (server-initiated SSE stream) ----------------------------

/// GET `/`: open the per-session SSE stream so the server can push
/// `notifications/*` and request responses to the client. The proxy holds
/// the stream open with periodic keepalives; data emission lands when
/// upstream notification forwarding is wired in a follow-up.
pub async fn handle_get(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    let session_id = match extract_session_id(&headers) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let session = match state.sessions.get(&session_id) {
        Some(s) => s,
        None => return (StatusCode::NOT_FOUND, "unknown session").into_response(),
    };

    let receiver = session.outbound.subscribe();
    let stream = BroadcastStream::new(receiver).filter_map(|result| async move {
        match result {
            Ok(value) => Event::default().json_data(value).ok().map(Ok::<_, Infallible>),
            // BroadcastStream surfaces a Lagged error if a slow consumer
            // missed events. We don't currently emit anything, so this is
            // never hit; if it ever fires we just skip the lagged window.
            Err(_) => None,
        }
    });

    let stream: Box<dyn Stream<Item = Result<Event, Infallible>> + Send + Unpin> =
        Box::new(Box::pin(stream));

    Sse::new(stream)
        .keep_alive(KeepAlive::new().interval(SSE_KEEP_ALIVE))
        .into_response()
}

// ---- Method handlers ------------------------------------------------------

async fn handle_initialize(
    state: &AppState,
    headers: &HeaderMap,
    request: JsonRpcRequest,
) -> Response {
    let specs = match parse_init_headers(headers) {
        Ok(s) => s,
        Err(BadInit::NotUtf8 { header }) => {
            return invalid_request_response(
                request.id,
                format!("{header} is not valid UTF-8"),
            );
        }
        Err(BadInit::NotJson { header, source }) => {
            return invalid_request_response(
                request.id,
                format!("{header} is not valid JSON: {source}"),
            );
        }
    };

    let connections = connect_all(&state.client, specs).await;
    let session_id = state.sessions.add(connections);

    let result = InitializeResult {
        protocol_version: PROTOCOL_VERSION.into(),
        capabilities: server_capabilities(),
        server_info: server_info(),
        instructions: Some(
            "ObjectiveAI MCP proxy. Multiplexes one client connection over many upstream MCP servers."
                .into(),
        ),
        _meta: None,
    };

    let body: JsonRpcResponse<InitializeResult> = JsonRpcResponse::Success {
        jsonrpc: "2.0".into(),
        id: request.id,
        result,
    };

    let mut headers = HeaderMap::new();
    let header_value = match HeaderValue::from_str(&session_id) {
        Ok(v) => v,
        Err(_) => {
            return internal_error_response(
                serde_json::Value::Null,
                format!("session id is not a valid header value: {session_id}"),
            );
        }
    };
    headers.insert(SESSION_ID_HEADER, header_value);

    (StatusCode::OK, headers, Json(body)).into_response()
}

fn handle_ping(request: JsonRpcRequest) -> Response {
    let body: JsonRpcResponse<serde_json::Value> = JsonRpcResponse::Success {
        jsonrpc: "2.0".into(),
        id: request.id,
        result: serde_json::json!({}),
    };
    (StatusCode::OK, Json(body)).into_response()
}

async fn handle_tools_list(
    sessions: &SessionManager,
    headers: &HeaderMap,
    request: JsonRpcRequest,
) -> Response {
    let session_id = match extract_session_id(headers) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let session = match sessions.get(&session_id) {
        Some(s) => s,
        None => return invalid_request_response(request.id, "unknown session".into()),
    };

    let result = session.list_tools().await;
    let body = JsonRpcResponse::Success {
        jsonrpc: "2.0".into(),
        id: request.id,
        result,
    };
    (StatusCode::OK, Json(body)).into_response()
}

async fn handle_tools_call(
    sessions: &SessionManager,
    headers: &HeaderMap,
    request: JsonRpcRequest,
) -> Response {
    let session_id = match extract_session_id(headers) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let session = match sessions.get(&session_id) {
        Some(s) => s,
        None => return invalid_request_response(request.id, "unknown session".into()),
    };

    let params: CallToolRequestParams = match request.params.clone() {
        Some(v) => match serde_json::from_value(v) {
            Ok(p) => p,
            Err(e) => {
                return invalid_params_response(
                    request.id,
                    format!("tools/call params: {e}"),
                );
            }
        },
        None => return invalid_params_response(request.id, "missing params".into()),
    };

    match session.call_tool(&params).await {
        Ok(result) => {
            let body = JsonRpcResponse::Success {
                jsonrpc: "2.0".into(),
                id: request.id,
                result,
            };
            (StatusCode::OK, Json(body)).into_response()
        }
        Err(CallToolError::ToolNotFound(name)) => {
            method_not_found_response(request.id, &format!("tool: {name}"))
        }
        Err(CallToolError::Upstream(e)) => {
            internal_error_response(request.id, format!("upstream call_tool: {e}"))
        }
    }
}

async fn handle_resources_list(
    sessions: &SessionManager,
    headers: &HeaderMap,
    request: JsonRpcRequest,
) -> Response {
    let session_id = match extract_session_id(headers) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let session = match sessions.get(&session_id) {
        Some(s) => s,
        None => return invalid_request_response(request.id, "unknown session".into()),
    };

    let result = session.list_resources().await;
    let body = JsonRpcResponse::Success {
        jsonrpc: "2.0".into(),
        id: request.id,
        result,
    };
    (StatusCode::OK, Json(body)).into_response()
}

async fn handle_resources_read(
    sessions: &SessionManager,
    headers: &HeaderMap,
    request: JsonRpcRequest,
) -> Response {
    let session_id = match extract_session_id(headers) {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let session = match sessions.get(&session_id) {
        Some(s) => s,
        None => return invalid_request_response(request.id, "unknown session".into()),
    };

    let params: ReadResourceRequestParams = match request.params.clone() {
        Some(v) => match serde_json::from_value(v) {
            Ok(p) => p,
            Err(e) => {
                return invalid_params_response(
                    request.id,
                    format!("resources/read params: {e}"),
                );
            }
        },
        None => return invalid_params_response(request.id, "missing params".into()),
    };

    match session.read_resource(&params.uri).await {
        Ok(result) => {
            let body = JsonRpcResponse::Success {
                jsonrpc: "2.0".into(),
                id: request.id,
                result,
            };
            (StatusCode::OK, Json(body)).into_response()
        }
        Err(ReadResourceError::ResourceNotFound(uri)) => {
            invalid_params_response(request.id, format!("resource not found: {uri}"))
        }
        Err(ReadResourceError::Upstream(e)) => {
            internal_error_response(request.id, format!("upstream read_resource: {e}"))
        }
    }
}

// ---- Helpers --------------------------------------------------------------

fn extract_session_id(headers: &HeaderMap) -> Result<String, Response> {
    match headers.get(SESSION_ID_HEADER) {
        Some(v) => match v.to_str() {
            Ok(s) => Ok(s.to_string()),
            Err(_) => Err(parse_error_response(format!(
                "{SESSION_ID_HEADER} is not valid UTF-8"
            ))),
        },
        None => Err(parse_error_response(format!(
            "missing {SESSION_ID_HEADER} header"
        ))),
    }
}

fn server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        experimental: None,
        logging: None,
        completions: None,
        prompts: None,
        // Tools and resources are exactly what `objectiveai::mcp::Connection`
        // exercises today. list_changed=true is honest about future
        // intent — the GET stream is open and will emit notifications
        // once upstream change-watch wiring lands.
        tools: Some(ToolsCapability {
            list_changed: Some(true),
        }),
        resources: Some(ResourcesCapability {
            subscribe: None,
            list_changed: Some(true),
        }),
        tasks: None,
    }
}

fn server_info() -> Implementation {
    Implementation {
        name: "objectiveai-mcp-proxy".into(),
        title: Some("ObjectiveAI MCP Proxy".into()),
        version: env!("CARGO_PKG_VERSION").into(),
        website_url: None,
        description: Some(
            "ObjectiveAI MCP proxy — multiplexes one client connection over many upstream MCP servers.".into(),
        ),
        icons: None,
    }
}

fn json_rpc_error_response(
    status: StatusCode,
    id: serde_json::Value,
    code: i64,
    message: String,
) -> Response {
    let body: JsonRpcResponse<()> = JsonRpcResponse::Error {
        jsonrpc: "2.0".into(),
        id,
        error: JsonRpcError {
            code,
            message,
            data: None,
        },
    };
    (status, Json(body)).into_response()
}

fn parse_error_response(message: String) -> Response {
    json_rpc_error_response(
        StatusCode::BAD_REQUEST,
        serde_json::Value::Null,
        PARSE_ERROR,
        message,
    )
}

fn invalid_request_response(id: serde_json::Value, message: String) -> Response {
    json_rpc_error_response(StatusCode::OK, id, INVALID_REQUEST, message)
}

fn invalid_params_response(id: serde_json::Value, message: String) -> Response {
    json_rpc_error_response(StatusCode::OK, id, INVALID_PARAMS, message)
}

fn internal_error_response(id: serde_json::Value, message: String) -> Response {
    json_rpc_error_response(StatusCode::OK, id, INTERNAL_ERROR, message)
}

fn method_not_found_response(id: serde_json::Value, method: &str) -> Response {
    json_rpc_error_response(
        StatusCode::OK,
        id,
        METHOD_NOT_FOUND,
        format!("method not found: {method}"),
    )
}

